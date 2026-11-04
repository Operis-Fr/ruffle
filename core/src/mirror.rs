//! Display tree mirror for embedders.
//!
//! When enabled, the mirror walks the display tree at the end of a tick (at a
//! bounded rate) and produces a [`MirrorFrame`]: the list of "useful" display
//! objects (interactive objects, text fields and identified containers) that
//! changed since the previous frame, with their Flex `id`, text and position in
//! CSS pixels. Embedders use it to build an accessible or testable view of what
//! the movie displays, e.g. a DOM overlay that end-to-end tests can target.
//!
//! Hooks in the player only set flags: no work is done per mutation, and
//! ActionScript getters are only called while emitting a frame, in a dedicated
//! activation.

use crate::avm2::object::TObject as _;
use crate::avm2::{Activation as Avm2Activation, Domain as Avm2Domain};
use crate::avm2::{Multiname, Object as Avm2Object, Value as Avm2Value};
use crate::context::UpdateContext;
use crate::display_object::{
    BoundsMode, DisplayObject, DisplayObjectPtr, TDisplayObject, TDisplayObjectContainer,
    TInteractiveObject,
};
use crate::events::MouseButton;
use crate::string::{AvmString, WStr};
use gc_arena::Collect;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use web_time::Instant;

/// Maximum length, in characters, of the strings read from ActionScript.
const MAX_STRING_LENGTH: usize = 500;

/// Bounds closer than this (in CSS pixels) are considered unchanged.
const BOUNDS_EPSILON: f64 = 0.5;

/// The kind of a mirrored display object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum MirrorKind {
    /// A container whose children receive mouse events (`mouseChildren`).
    Container,
    /// A button, or a container that handles the mouse as a whole
    /// (`mouseChildren = false`, as Flex buttons do).
    Interactive,
    /// A text field (`EditText`).
    Text,
    /// Anything else (shapes, bitmaps, static text...).
    Other,
}

/// A rectangle in CSS pixels, relative to the top-left corner of the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MirrorRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl MirrorRect {
    fn approx_eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < BOUNDS_EPSILON
            && (self.y - other.y).abs() < BOUNDS_EPSILON
            && (self.width - other.width).abs() < BOUNDS_EPSILON
            && (self.height - other.height).abs() < BOUNDS_EPSILON
    }
}

/// A mirrored display object.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MirrorNode {
    pub uid: u64,
    /// uid of the nearest *emitted* ancestor; `None` for the root (stage children).
    pub parent: Option<u64>,
    /// Position among the parent's children in the render list
    /// (all children, not only emitted ones).
    pub index: u32,
    pub kind: MirrorKind,
    /// AVM2 qualified name, e.g. `mx.controls::TextInput`;
    /// the display object type when there is no AVM2 object.
    pub class_name: String,
    /// `DisplayObject.name`.
    pub name: Option<String>,
    /// Flex `UIComponent.id`.
    pub id: Option<String>,
    pub automation_name: Option<String>,
    pub label: Option<String>,
    /// `None` for password fields (`TextField.displayAsPassword`, or the
    /// `displayAsPassword` property of a component).
    pub text: Option<String>,
    pub tool_tip: Option<String>,
    pub enabled: Option<bool>,
    pub editable: Option<bool>,
    pub selected_index: Option<i32>,
    pub item_index: Option<i32>,
    /// Own flag and every ancestor visible.
    pub visible: bool,
    /// Whether this node, or a non-emitted descendant, has the focus.
    pub focused: bool,
    pub bounds: MirrorRect,
}

impl MirrorNode {
    /// Whether the receiver needs to be told about `self`, `previous` being
    /// the version it already knows. Bounds are compared with a tolerance.
    pub fn differs_from(&self, previous: &MirrorNode) -> bool {
        self.uid != previous.uid
            || self.parent != previous.parent
            || self.index != previous.index
            || self.kind != previous.kind
            || self.class_name != previous.class_name
            || self.name != previous.name
            || self.id != previous.id
            || self.automation_name != previous.automation_name
            || self.label != previous.label
            || self.text != previous.text
            || self.tool_tip != previous.tool_tip
            || self.enabled != previous.enabled
            || self.editable != previous.editable
            || self.selected_index != previous.selected_index
            || self.item_index != previous.item_index
            || self.visible != previous.visible
            || self.focused != previous.focused
            || !self.bounds.approx_eq(&previous.bounds)
    }
}

/// A mouse button, as reported in [`MirrorEvent::Click`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[collect(require_static)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum MirrorButton {
    Left,
    Right,
    Middle,
}

/// Something the user (or an injected event) did, resolved to a mirrored node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum MirrorEvent {
    Click { uid: u64, button: MirrorButton },
    TextInput { uid: u64 },
    Focus { uid: Option<u64> },
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MirrorStats {
    /// Display objects visited by the walk.
    pub nodes_walked: u32,
    /// Display objects emitted by the filter (changed or not).
    pub nodes_emitted: u32,
    /// ActionScript getters called.
    pub getters_called: u32,
    /// Duration of the walk, in microseconds.
    pub elapsed_us: u64,
    /// Whether `max_nodes` was reached.
    pub truncated: bool,
}

/// The changes of the mirror since the previous frame.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MirrorFrame {
    pub frame: u64,
    /// `true` when the receiver must drop everything it knows
    /// (mirror enabled, refresh, filter change).
    pub reset: bool,
    /// New or changed nodes, parents before children.
    pub upserts: Vec<MirrorNode>,
    /// uids of the nodes that are no longer emitted.
    pub removed: Vec<u64>,
    pub events: Vec<MirrorEvent>,
    pub stats: MirrorStats,
}

/// A glob pattern: `*` matches any sequence of characters, `?` any character.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Pattern(String);

impl Pattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    pub fn matches(&self, text: &str) -> bool {
        let pattern: Vec<char> = self.0.chars().collect();
        let text: Vec<char> = text.chars().collect();
        let (mut p, mut t) = (0, 0);
        // Position of the last `*` in the pattern, and of the text it was tried at.
        let mut backtrack: Option<(usize, usize)> = None;
        while t < text.len() {
            if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) {
                p += 1;
                t += 1;
            } else if p < pattern.len() && pattern[p] == '*' {
                backtrack = Some((p, t));
                p += 1;
            } else if let Some((star, start)) = backtrack {
                // Let the last `*` swallow one more character.
                p = star + 1;
                t = start + 1;
                backtrack = Some((star, start + 1));
            } else {
                return false;
            }
        }
        pattern[p..].iter().all(|c| *c == '*')
    }
}

fn matches_any(patterns: &[Pattern], text: &str) -> bool {
    patterns.iter().any(|pattern| pattern.matches(text))
}

/// Which display objects the mirror emits.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "camelCase"))]
pub struct MirrorFilter {
    /// Emit buttons and containers that handle the mouse as a whole.
    pub interactive: bool,
    /// Emit text fields (`EditText`), and components with a `text`
    /// property (e.g. Flex `TextInput`, `Label`).
    pub text: bool,
    /// Emit any object whose Flex `id` is set, and list item renderers,
    /// even if neither interactive nor text.
    pub identified: bool,
    /// Emit objects whose visibility (or an ancestor's) is off.
    /// When `false`, only the root of a hidden subtree is emitted.
    pub emit_hidden: bool,
    /// Glob patterns on the qualified class name, e.g. `mx.skins::*`.
    pub exclude_classes: Vec<Pattern>,
    /// Glob patterns on the Flex id.
    pub exclude_ids: Vec<Pattern>,
    /// Glob patterns on the instance name.
    pub exclude_names: Vec<Pattern>,
    /// Beyond this number of nodes, stop emitting and set `stats.truncated`.
    pub max_nodes: u32,
}

impl Default for MirrorFilter {
    fn default() -> Self {
        Self {
            interactive: true,
            text: true,
            identified: true,
            emit_hidden: false,
            exclude_classes: Vec::new(),
            exclude_ids: Vec::new(),
            exclude_names: Vec::new(),
            max_nodes: 5000,
        }
    }
}

/// Properties read from ActionScript getters.
#[derive(Clone, Debug, Default, PartialEq)]
struct Avm2Props {
    id: Option<String>,
    automation_name: Option<String>,
    label: Option<String>,
    text: Option<String>,
    tool_tip: Option<String>,
    enabled: Option<bool>,
    editable: Option<bool>,
    selected_index: Option<i32>,
    item_index: Option<i32>,
}

/// What the mirror knows about a display object it has met.
#[derive(Debug)]
struct TrackedNode {
    /// `Some` while the object is emitted.
    uid: Option<u64>,
    /// Pass during which the object was last visited.
    pass: u64,
    class_name: String,
    class_excluded: bool,
    /// Whether the class looks like a list item renderer.
    item_renderer: bool,
    /// Whether the class is a component with a `text` property (other than
    /// a text field), e.g. a Flex `TextInput` or `Label`.
    text_component: bool,
    /// Whether `props.id` has been read.
    identity_read: bool,
    /// Whether all of `props` has been read.
    props_read: bool,
    props: Avm2Props,
    /// The node as last sent to the receiver.
    last: Option<MirrorNode>,
}

/// The display-object independent part of the mirror: identity of the
/// tracked objects and allocation of uids.
#[derive(Debug, Default)]
struct Tracker {
    nodes: HashMap<*const DisplayObjectPtr, TrackedNode>,
    next_uid: u64,
    pass: u64,
    emitted: u32,
    truncated: bool,
}

impl Tracker {
    fn begin_pass(&mut self) {
        self.pass += 1;
        self.emitted = 0;
        self.truncated = false;
    }

    /// Marks `ptr` as visited during this pass, creating its entry with
    /// `make` on first encounter. Returns the entry and whether it is new.
    fn visit(
        &mut self,
        ptr: *const DisplayObjectPtr,
        make: impl FnOnce() -> (String, bool),
    ) -> (&mut TrackedNode, bool) {
        let pass = self.pass;
        let mut is_new = false;
        let node = self.nodes.entry(ptr).or_insert_with(|| {
            is_new = true;
            let (class_name, class_excluded) = make();
            TrackedNode {
                uid: None,
                pass,
                class_name,
                class_excluded,
                item_renderer: false,
                text_component: false,
                identity_read: false,
                props_read: false,
                props: Avm2Props::default(),
                last: None,
            }
        });
        node.pass = pass;
        (node, is_new)
    }

    /// Tries to emit `ptr` during this pass. Returns its uid (a new one if it
    /// was not emitted before), or `None` if `max_nodes` is reached; the
    /// uid that `ptr` had is then pushed to `removed`.
    fn emit(
        &mut self,
        ptr: *const DisplayObjectPtr,
        max_nodes: u32,
        removed: &mut Vec<u64>,
    ) -> Option<u64> {
        if self.emitted >= max_nodes {
            self.truncated = true;
            self.withhold(ptr, removed);
            return None;
        }
        self.emitted += 1;
        let next_uid = &mut self.next_uid;
        let node = self.nodes.get_mut(&ptr)?;
        Some(*node.uid.get_or_insert_with(|| {
            *next_uid += 1;
            *next_uid
        }))
    }

    /// `ptr` is visited but not emitted during this pass.
    fn withhold(&mut self, ptr: *const DisplayObjectPtr, removed: &mut Vec<u64>) {
        if let Some(node) = self.nodes.get_mut(&ptr)
            && let Some(uid) = node.uid.take()
        {
            node.last = None;
            removed.push(uid);
        }
    }

    /// Forgets the objects that were not visited during this pass.
    fn end_pass(&mut self, removed: &mut Vec<u64>) {
        let pass = self.pass;
        self.nodes.retain(|_, node| {
            if node.pass != pass
                && let Some(uid) = node.uid
            {
                removed.push(uid);
            }
            node.pass == pass
        });
    }

    fn uid(&self, ptr: *const DisplayObjectPtr) -> Option<u64> {
        self.nodes.get(&ptr).and_then(|node| node.uid)
    }
}

/// An event waiting for the next frame to be resolved to a uid.
#[derive(Collect)]
#[collect(no_drop)]
enum PendingEvent<'gc> {
    Click(DisplayObject<'gc>, MirrorButton),
    TextInput(DisplayObject<'gc>),
    Focus(Option<DisplayObject<'gc>>),
}

/// A display object visited by the walk.
struct Visit<'gc> {
    object: DisplayObject<'gc>,
    parent: Option<usize>,
    index: u32,
    visible: bool,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct Mirror<'gc> {
    enabled: bool,
    #[collect(require_static)]
    filter: MirrorFilter,
    #[collect(require_static)]
    tracker: Tracker,
    /// The objects known by `tracker`, kept alive so that a pointer is never
    /// reused by another object while tracked.
    tracked_objects: Vec<DisplayObject<'gc>>,
    frame_counter: u64,
    /// Something was added, removed, shown, hidden, focused or resized.
    structure_dirty: bool,
    /// The receiver must drop everything it knows at the next frame.
    reset_pending: bool,
    /// Text fields whose text changed.
    dirty_objects: Vec<DisplayObject<'gc>>,
    #[collect(require_static)]
    dirty_ptrs: HashSet<*const DisplayObjectPtr>,
    pending_events: Vec<PendingEvent<'gc>>,
    #[collect(require_static)]
    last_emit: Option<Instant>,
    #[collect(require_static)]
    last_properties_refresh: Option<Instant>,
    #[collect(require_static)]
    min_interval: Duration,
    /// Period of the slow refresh of ActionScript properties; zero means never.
    #[collect(require_static)]
    properties_interval: Duration,
    /// Getter failures already logged, by (class, property).
    #[collect(require_static)]
    logged_errors: HashSet<(String, &'static str)>,
}

impl Default for Mirror<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> Mirror<'gc> {
    pub fn new() -> Self {
        Self {
            enabled: false,
            filter: MirrorFilter::default(),
            tracker: Tracker::default(),
            tracked_objects: Vec::new(),
            frame_counter: 0,
            structure_dirty: false,
            reset_pending: false,
            dirty_objects: Vec::new(),
            dirty_ptrs: HashSet::new(),
            pending_events: Vec::new(),
            last_emit: None,
            last_properties_refresh: None,
            min_interval: Duration::from_millis(100),
            properties_interval: Duration::from_secs(1),
            logged_errors: HashSet::new(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        self.forget();
        self.reset_pending = enabled;
    }

    pub fn set_filter(&mut self, filter: MirrorFilter) {
        self.filter = filter;
        // Cached decisions (class exclusion, identity) depend on the filter.
        self.forget();
        self.reset_pending = self.enabled;
    }

    pub fn set_intervals(&mut self, min_interval: Duration, properties_interval: Duration) {
        self.min_interval = min_interval;
        self.properties_interval = properties_interval;
    }

    /// Drops everything known about the display tree.
    fn forget(&mut self) {
        self.tracker.nodes.clear();
        self.tracked_objects.clear();
        self.dirty_objects.clear();
        self.dirty_ptrs.clear();
        self.pending_events.clear();
        self.structure_dirty = false;
        self.last_emit = None;
        self.last_properties_refresh = None;
    }

    /// Something was added, removed, shown, hidden, focused or resized.
    #[inline]
    pub fn mark_structure(&mut self) {
        if self.enabled {
            self.structure_dirty = true;
        }
    }

    /// The text of `object` changed.
    #[inline]
    pub fn mark_text(&mut self, object: DisplayObject<'gc>) {
        if self.enabled && self.dirty_ptrs.insert(object.as_ptr()) {
            self.dirty_objects.push(object);
        }
    }

    /// The user pressed a mouse button on `object`.
    #[inline]
    pub fn push_click(&mut self, object: DisplayObject<'gc>, button: MouseButton) {
        if !self.enabled {
            return;
        }
        let button = match button {
            MouseButton::Left => MirrorButton::Left,
            MouseButton::Right => MirrorButton::Right,
            MouseButton::Middle => MirrorButton::Middle,
            MouseButton::Unknown => return,
        };
        self.pending_events
            .push(PendingEvent::Click(object, button));
    }

    /// The user typed text in `object`.
    #[inline]
    pub fn push_text_input(&mut self, object: DisplayObject<'gc>) {
        if self.enabled {
            self.pending_events.push(PendingEvent::TextInput(object));
        }
    }

    /// The focus moved to `object`.
    #[inline]
    pub fn push_focus(&mut self, object: Option<DisplayObject<'gc>>) {
        if self.enabled {
            self.structure_dirty = true;
            self.pending_events.push(PendingEvent::Focus(object));
        }
    }

    /// Walks the display tree and returns the changes since the previous
    /// frame, if a frame is due. `force` ignores the minimal interval, reads
    /// every property again and asks the receiver to reset.
    pub fn flush(context: &mut UpdateContext<'gc>, force: bool) -> Option<MirrorFrame> {
        let mirror = &mut *context.mirror;
        if !mirror.enabled {
            return None;
        }
        let now = Instant::now();
        if !force
            && mirror
                .last_emit
                .is_some_and(|last| now.duration_since(last) < mirror.min_interval)
        {
            return None;
        }
        let properties_due = force
            || (!mirror.properties_interval.is_zero()
                && mirror
                    .last_properties_refresh
                    .is_none_or(|last| now.duration_since(last) >= mirror.properties_interval));
        let reset = force || mirror.reset_pending;
        if !reset
            && !properties_due
            && !mirror.structure_dirty
            && mirror.dirty_objects.is_empty()
            && mirror.pending_events.is_empty()
        {
            return None;
        }

        // Take what the hooks recorded: getters called below may record more,
        // which will be handled by the next frame.
        mirror.structure_dirty = false;
        mirror.reset_pending = false;
        mirror.dirty_ptrs.clear();
        let dirty_objects = std::mem::take(&mut mirror.dirty_objects);
        let pending_events = std::mem::take(&mut mirror.pending_events);
        let filter = mirror.filter.clone();
        let mut tracker = std::mem::take(&mut mirror.tracker);
        let logged_errors = std::mem::take(&mut mirror.logged_errors);

        // Text changes are reported on the nearest emitted node, and on the text
        // components that contain it, so that e.g. a Flex `TextInput` sees the
        // change of its inner text field.
        let mut dirty_text: HashSet<*const DisplayObjectPtr> = HashSet::new();
        for object in &dirty_objects {
            let mut current = Some(*object);
            let mut found = false;
            while let Some(object) = current {
                let ptr = object.as_ptr();
                if let Some(node) = tracker.nodes.get(&ptr)
                    && node.uid.is_some()
                {
                    if found && !node.text_component {
                        break;
                    }
                    dirty_text.insert(ptr);
                    found = true;
                }
                current = object.parent();
            }
        }

        let mut visits = Vec::with_capacity(tracker.nodes.len());
        if let Some(stage) = DisplayObject::Stage(context.stage).as_container() {
            for (index, child) in stage.iter_render_list().enumerate() {
                walk(
                    child,
                    None,
                    index as u32,
                    true,
                    filter.emit_hidden,
                    &mut visits,
                );
            }
        }

        let mut reader = PropertyReader::new(context, logged_errors);
        let mut removed = Vec::new();
        let mut emitted: Vec<Option<u64>> = Vec::with_capacity(visits.len());
        let mut parents: Vec<Option<u64>> = Vec::with_capacity(visits.len());
        let mut nodes: Vec<(*const DisplayObjectPtr, MirrorNode)> = Vec::new();
        tracker.begin_pass();
        for visit in &visits {
            let object = visit.object;
            let ptr = object.as_ptr();
            let parent = visit
                .parent
                .and_then(|parent| emitted[parent].or(parents[parent]));
            parents.push(parent);

            let (node, is_new) = tracker.visit(ptr, || {
                let class_name = class_name(reader.context, object);
                let excluded = matches_any(&filter.exclude_classes, &class_name);
                (class_name, excluded)
            });
            if is_new && !node.class_excluded {
                node.item_renderer = reader.is_item_renderer(object);
                node.text_component = reader.is_text_component(object);
            }
            let kind = kind_of(object);
            let name = object.name().map(|name| name.to_string());

            let mut candidate = !node.class_excluded
                && match kind {
                    MirrorKind::Text => filter.text,
                    MirrorKind::Interactive => {
                        filter.interactive || filter.text && node.text_component
                    }
                    MirrorKind::Container | MirrorKind::Other => filter.text && node.text_component,
                };
            if !node.class_excluded
                && !node.identity_read
                && (filter.identified && !candidate || candidate && !filter.exclude_ids.is_empty())
            {
                node.props.id = reader.read_string(object, &node.class_name, "id");
                node.identity_read = true;
            }
            if !candidate && !node.class_excluded && filter.identified {
                candidate = node.props.id.is_some() || node.item_renderer;
            }
            if candidate {
                candidate = !node
                    .props
                    .id
                    .as_deref()
                    .is_some_and(|id| matches_any(&filter.exclude_ids, id))
                    && !name
                        .as_deref()
                        .is_some_and(|name| matches_any(&filter.exclude_names, name));
            }

            let uid = if candidate {
                tracker.emit(ptr, filter.max_nodes, &mut removed)
            } else {
                tracker.withhold(ptr, &mut removed);
                None
            };
            emitted.push(uid);
            let Some(uid) = uid else {
                continue;
            };

            let Some(node) = tracker.nodes.get_mut(&ptr) else {
                continue;
            };
            if !node.props_read || properties_due {
                node.props = reader.read_props(object, &node.class_name, node.item_renderer);
                node.identity_read = true;
                node.props_read = true;
            } else if dirty_text.contains(&ptr) {
                node.props.text = reader.read_text(object, &node.class_name);
            }

            let editable = match object.as_edit_text() {
                Some(text) => Some(text.is_editable()),
                None => node.props.editable,
            };
            // The text of a password field is never mirrored.
            let text = match object.as_edit_text() {
                Some(text) if text.is_password() => None,
                Some(text) => Some(truncate(text.text().to_string())),
                None => node.props.text.clone(),
            };
            nodes.push((
                ptr,
                MirrorNode {
                    uid,
                    parent,
                    index: visit.index,
                    kind,
                    class_name: node.class_name.clone(),
                    name,
                    id: node.props.id.clone(),
                    automation_name: node.props.automation_name.clone(),
                    label: node.props.label.clone(),
                    text,
                    tool_tip: node.props.tool_tip.clone(),
                    enabled: node.props.enabled,
                    editable,
                    selected_index: node.props.selected_index,
                    item_index: node.props.item_index,
                    visible: visit.visible,
                    focused: false,
                    bounds: bounds_of(reader.context, object),
                },
            ));
        }
        let PropertyReader {
            context,
            logged_errors,
            getters_called,
            ..
        } = reader;
        tracker.end_pass(&mut removed);

        let focused = context
            .focus_tracker
            .get()
            .and_then(|focus| nearest_emitted(&tracker, focus.as_displayobject()))
            .map(|(_, uid)| uid);
        let mut upserts = Vec::new();
        for (ptr, mut node) in nodes {
            node.focused = Some(node.uid) == focused;
            let Some(tracked) = tracker.nodes.get_mut(&ptr) else {
                continue;
            };
            if reset
                || tracked
                    .last
                    .as_ref()
                    .is_none_or(|last| node.differs_from(last))
            {
                upserts.push(node.clone());
            }
            tracked.last = Some(node);
        }

        let events = pending_events
            .into_iter()
            .filter_map(|event| match event {
                PendingEvent::Click(object, button) => nearest_emitted(&tracker, object)
                    .map(|(_, uid)| MirrorEvent::Click { uid, button }),
                PendingEvent::TextInput(object) => {
                    nearest_emitted(&tracker, object).map(|(_, uid)| MirrorEvent::TextInput { uid })
                }
                PendingEvent::Focus(None) => Some(MirrorEvent::Focus { uid: None }),
                PendingEvent::Focus(Some(object)) => nearest_emitted(&tracker, object)
                    .map(|(_, uid)| MirrorEvent::Focus { uid: Some(uid) }),
            })
            .collect();

        let mirror = &mut *context.mirror;
        let stats = MirrorStats {
            nodes_walked: visits.len() as u32,
            nodes_emitted: tracker.emitted,
            getters_called,
            elapsed_us: now.elapsed().as_micros() as u64,
            truncated: tracker.truncated,
        };
        if stats.truncated {
            tracing::warn!(
                "Display tree mirror truncated to {} nodes",
                filter.max_nodes
            );
        }
        mirror.tracked_objects = visits.into_iter().map(|visit| visit.object).collect();
        mirror.tracker = tracker;
        mirror.logged_errors = logged_errors;
        mirror.frame_counter += 1;
        mirror.last_emit = Some(now);
        if properties_due {
            mirror.last_properties_refresh = Some(now);
        }
        Some(MirrorFrame {
            frame: mirror.frame_counter,
            reset,
            upserts,
            removed: if reset { Vec::new() } else { removed },
            events,
            stats,
        })
    }
}

/// Depth-first walk in render order, skipping the children of hidden objects
/// unless `emit_hidden`.
fn walk<'gc>(
    object: DisplayObject<'gc>,
    parent: Option<usize>,
    index: u32,
    parent_visible: bool,
    emit_hidden: bool,
    visits: &mut Vec<Visit<'gc>>,
) {
    let visible = parent_visible && object.visible();
    let this = visits.len();
    visits.push(Visit {
        object,
        parent,
        index,
        visible,
    });
    if !visible && !emit_hidden {
        return;
    }
    if let Some(container) = object.as_container() {
        for (index, child) in container.iter_render_list().enumerate() {
            walk(
                child,
                Some(this),
                index as u32,
                visible,
                emit_hidden,
                visits,
            );
        }
    }
}

/// The nearest emitted node among `object` and its ancestors.
fn nearest_emitted(
    tracker: &Tracker,
    object: DisplayObject<'_>,
) -> Option<(*const DisplayObjectPtr, u64)> {
    let mut current = Some(object);
    while let Some(object) = current {
        if let Some(uid) = tracker.uid(object.as_ptr()) {
            return Some((object.as_ptr(), uid));
        }
        current = object.parent();
    }
    None
}

fn kind_of(object: DisplayObject<'_>) -> MirrorKind {
    match object {
        DisplayObject::EditText(_) => MirrorKind::Text,
        DisplayObject::Avm1Button(_) | DisplayObject::Avm2Button(_) => MirrorKind::Interactive,
        DisplayObject::MovieClip(_) | DisplayObject::LoaderDisplay(_) => {
            match object.as_container() {
                Some(container) if !container.raw_container().mouse_children() => {
                    MirrorKind::Interactive
                }
                _ => MirrorKind::Container,
            }
        }
        _ => MirrorKind::Other,
    }
}

fn class_name<'gc>(context: &mut UpdateContext<'gc>, object: DisplayObject<'gc>) -> String {
    if let Some(object) = object.object2() {
        return Avm2Object::from(object)
            .instance_of_class_name(context.gc())
            .to_string();
    }
    match object {
        DisplayObject::Stage(_) => "Stage",
        DisplayObject::Bitmap(_) => "Bitmap",
        DisplayObject::Avm1Button(_) => "Avm1Button",
        DisplayObject::Avm2Button(_) => "Avm2Button",
        DisplayObject::EditText(_) => "EditText",
        DisplayObject::TextLine(_) => "TextLine",
        DisplayObject::Graphic(_) => "Graphic",
        DisplayObject::MorphShape(_) => "MorphShape",
        DisplayObject::MovieClip(_) => "MovieClip",
        DisplayObject::Text(_) => "Text",
        DisplayObject::Video(_) => "Video",
        DisplayObject::LoaderDisplay(_) => "LoaderDisplay",
    }
    .to_string()
}

/// World bounds in CSS pixels, relative to the top-left corner of the viewport.
fn bounds_of<'gc>(context: &mut UpdateContext<'gc>, object: DisplayObject<'gc>) -> MirrorRect {
    let matrix = context.stage.view_matrix() * context.stage.base().matrix();
    let bounds = matrix * object.world_bounds(BoundsMode::Engine);
    if !bounds.is_valid() {
        return MirrorRect::default();
    }
    let scale = context.renderer.viewport_dimensions().scale_factor;
    let scale = if scale > 0.0 { scale } else { 1.0 };
    MirrorRect {
        x: bounds.x_min.to_pixels() / scale,
        y: bounds.y_min.to_pixels() / scale,
        width: bounds.width().to_pixels() / scale,
        height: bounds.height().to_pixels() / scale,
    }
}

fn truncate(mut text: String) -> String {
    if let Some((index, _)) = text.char_indices().nth(MAX_STRING_LENGTH) {
        text.truncate(index);
    }
    text
}

/// Reads public properties of display objects through their getters.
struct PropertyReader<'a, 'gc> {
    context: &'a mut UpdateContext<'gc>,
    domain: Option<Avm2Domain<'gc>>,
    logged_errors: HashSet<(String, &'static str)>,
    getters_called: u32,
}

impl<'a, 'gc> PropertyReader<'a, 'gc> {
    fn new(
        context: &'a mut UpdateContext<'gc>,
        logged_errors: HashSet<(String, &'static str)>,
    ) -> Self {
        let domain = context
            .library
            .library_for_movie(*context.root_swf)
            .map(|library| library.avm2_domain());
        Self {
            context,
            domain,
            logged_errors,
            getters_called: 0,
        }
    }

    fn name(&mut self, name: &'static str) -> AvmString<'gc> {
        self.context
            .strings
            .intern_static(WStr::from_units(name.as_bytes()))
            .into()
    }

    fn has_trait(&mut self, object: Avm2Object<'gc>, name: &'static str) -> bool {
        let namespace = self.context.avm2.find_public_namespace();
        let name = self.name(name);
        object.vtable().has_trait(&Multiname::new(namespace, name))
    }

    /// Calls the getter of the public property `name` if the class of
    /// `object` declares it. Errors are logged once per class and property.
    fn read(
        &mut self,
        object: Avm2Object<'gc>,
        class_name: &str,
        name: &'static str,
    ) -> Option<Avm2Value<'gc>> {
        let domain = self.domain?;
        if !self.has_trait(object, name) {
            return None;
        }
        self.getters_called += 1;
        let property = self.name(name);
        let mut activation = Avm2Activation::from_domain(self.context, domain);
        match Avm2Value::from(object).get_public_property(property, &mut activation) {
            Ok(Avm2Value::Null | Avm2Value::Undefined) => None,
            Ok(value) => Some(value),
            Err(error) => {
                if self.logged_errors.insert((class_name.to_string(), name)) {
                    tracing::debug!("Mirror: error reading {class_name}.{name}: {error:?}");
                }
                None
            }
        }
    }

    fn read_string(
        &mut self,
        object: DisplayObject<'gc>,
        class_name: &str,
        name: &'static str,
    ) -> Option<String> {
        let object = Avm2Object::from(object.object2()?);
        let value = self.read(object, class_name, name)?;
        let domain = self.domain?;
        let mut activation = Avm2Activation::from_domain(self.context, domain);
        let string = value.coerce_to_string(&mut activation).ok()?;
        Some(truncate(string.to_string())).filter(|string| !string.is_empty() || name == "text")
    }

    fn read_bool(
        &mut self,
        object: DisplayObject<'gc>,
        class_name: &str,
        name: &'static str,
    ) -> Option<bool> {
        let object = Avm2Object::from(object.object2()?);
        Some(self.read(object, class_name, name)?.coerce_to_boolean())
    }

    fn read_i32(
        &mut self,
        object: Avm2Object<'gc>,
        class_name: &str,
        name: &'static str,
    ) -> Option<i32> {
        let value = self.read(object, class_name, name)?;
        let domain = self.domain?;
        let mut activation = Avm2Activation::from_domain(self.context, domain);
        value.coerce_to_i32(&mut activation).ok()
    }

    /// A component with a `text` property, so that e.g. a Flex `TextInput`
    /// without `id` is emitted itself, rather than only its inner text field.
    fn is_text_component(&mut self, object: DisplayObject<'gc>) -> bool {
        if object.as_edit_text().is_some() {
            return false;
        }
        let Some(object) = object.object2() else {
            return false;
        };
        self.has_trait(Avm2Object::from(object), "text")
    }

    fn is_item_renderer(&mut self, object: DisplayObject<'gc>) -> bool {
        let Some(object) = object.object2() else {
            return false;
        };
        let object = Avm2Object::from(object);
        self.has_trait(object, "itemIndex") || self.has_trait(object, "listData")
    }

    /// `itemIndex` (Spark item renderers), or `listData.rowIndex` (mx).
    fn read_item_index(&mut self, object: DisplayObject<'gc>, class_name: &str) -> Option<i32> {
        let object = Avm2Object::from(object.object2()?);
        if self.has_trait(object, "itemIndex") {
            return self.read_i32(object, class_name, "itemIndex");
        }
        let list_data = self.read(object, class_name, "listData")?.as_object()?;
        self.read_i32(list_data, class_name, "rowIndex")
    }

    fn read_text(&mut self, object: DisplayObject<'gc>, class_name: &str) -> Option<String> {
        // The text of an `EditText` is read directly.
        if object.as_edit_text().is_some() {
            return None;
        }
        // The text of a password field is never mirrored.
        if self.read_bool(object, class_name, "displayAsPassword") == Some(true) {
            return None;
        }
        self.read_string(object, class_name, "text")
    }

    fn read_props(
        &mut self,
        object: DisplayObject<'gc>,
        class_name: &str,
        item_renderer: bool,
    ) -> Avm2Props {
        if object.object2().is_none() {
            return Avm2Props::default();
        }
        let object_is_text = object.as_edit_text().is_some();
        Avm2Props {
            id: self.read_string(object, class_name, "id"),
            automation_name: self.read_string(object, class_name, "automationName"),
            label: self.read_string(object, class_name, "label"),
            text: self.read_text(object, class_name),
            tool_tip: self.read_string(object, class_name, "toolTip"),
            enabled: self.read_bool(object, class_name, "enabled"),
            editable: if object_is_text {
                None
            } else {
                self.read_bool(object, class_name, "editable")
            },
            selected_index: object
                .object2()
                .and_then(|o| self.read_i32(Avm2Object::from(o), class_name, "selectedIndex")),
            item_index: if item_renderer {
                self.read_item_index(object, class_name)
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ptr(address: usize) -> *const DisplayObjectPtr {
        address as *const DisplayObjectPtr
    }

    fn node(uid: u64) -> MirrorNode {
        MirrorNode {
            uid,
            parent: None,
            index: 0,
            kind: MirrorKind::Interactive,
            class_name: "mx.controls::Button".to_string(),
            name: None,
            id: Some("btnOk".to_string()),
            automation_name: None,
            label: Some("OK".to_string()),
            text: None,
            tool_tip: None,
            enabled: Some(true),
            editable: None,
            selected_index: None,
            item_index: None,
            visible: true,
            focused: false,
            bounds: MirrorRect {
                x: 10.0,
                y: 20.0,
                width: 80.0,
                height: 22.0,
            },
        }
    }

    /// Runs a pass where `visited` objects are visited and emitted.
    fn pass(
        tracker: &mut Tracker,
        visited: &[usize],
        max_nodes: u32,
    ) -> (Vec<Option<u64>>, Vec<u64>) {
        let mut removed = Vec::new();
        tracker.begin_pass();
        let uids = visited
            .iter()
            .map(|address| {
                tracker.visit(ptr(*address), || ("Sprite".to_string(), false));
                tracker.emit(ptr(*address), max_nodes, &mut removed)
            })
            .collect();
        tracker.end_pass(&mut removed);
        (uids, removed)
    }

    #[test]
    fn pattern_matches_globs() {
        assert!(Pattern::new("mx.skins::*").matches("mx.skins::ProgrammaticSkin"));
        assert!(Pattern::new("mx.skins::*").matches("mx.skins::"));
        assert!(!Pattern::new("mx.skins::*").matches("mx.controls::Button"));
        assert!(Pattern::new("*Skin").matches("spark.skins.spark::ButtonSkin"));
        assert!(!Pattern::new("*Skin").matches("spark.skins.spark::ButtonSkinPart"));
        assert!(Pattern::new("*::*Text*").matches("mx.core::UITextField"));
        assert!(Pattern::new("C_?").matches("C_1"));
        assert!(!Pattern::new("C_?").matches("C_12"));
        assert!(Pattern::new("C_*_?").matches("C_NOM_X"));
        assert!(Pattern::new("*").matches(""));
        assert!(!Pattern::new("").matches("a"));
        assert!(Pattern::new("flash.display::Shape").matches("flash.display::Shape"));
        assert!(!Pattern::new("flash.display::Shape").matches("flash.display::Sprite"));
        assert!(Pattern::new("é*").matches("été"));
    }

    #[test]
    fn node_diff_tolerates_half_a_pixel() {
        let previous = node(1);
        let mut current = node(1);
        current.bounds.x += 0.4;
        current.bounds.height -= 0.3;
        assert!(!current.differs_from(&previous));

        current.bounds.y += 0.6;
        assert!(current.differs_from(&previous));

        let mut current = node(1);
        current.text = Some(String::new());
        assert!(current.differs_from(&previous));

        let mut current = node(1);
        current.focused = true;
        assert!(current.differs_from(&previous));
    }

    #[test]
    fn uids_are_given_and_taken_back() {
        let mut tracker = Tracker::default();

        let (uids, removed) = pass(&mut tracker, &[0x10, 0x20], 100);
        assert_eq!(uids, vec![Some(1), Some(2)]);
        assert!(removed.is_empty());

        // Unchanged objects keep their uid.
        let (uids, removed) = pass(&mut tracker, &[0x10, 0x20], 100);
        assert_eq!(uids, vec![Some(1), Some(2)]);
        assert!(removed.is_empty());

        // An object leaving the stage is removed and forgotten.
        let (uids, removed) = pass(&mut tracker, &[0x10], 100);
        assert_eq!(uids, vec![Some(1)]);
        assert_eq!(removed, vec![2]);
        assert!(!tracker.nodes.contains_key(&ptr(0x20)));

        // Coming back, it gets a new uid.
        let (uids, removed) = pass(&mut tracker, &[0x10, 0x20], 100);
        assert_eq!(uids, vec![Some(1), Some(3)]);
        assert!(removed.is_empty());
    }

    #[test]
    fn withheld_objects_lose_their_uid() {
        let mut tracker = Tracker::default();
        pass(&mut tracker, &[0x10], 100);

        let mut removed = Vec::new();
        tracker.begin_pass();
        tracker.visit(ptr(0x10), || unreachable!());
        tracker.withhold(ptr(0x10), &mut removed);
        tracker.end_pass(&mut removed);
        assert_eq!(removed, vec![1]);
        // Still tracked (its cached properties are kept), but not emitted.
        assert!(tracker.nodes.contains_key(&ptr(0x10)));
        assert_eq!(tracker.uid(ptr(0x10)), None);

        let (uids, _) = pass(&mut tracker, &[0x10], 100);
        assert_eq!(uids, vec![Some(2)]);
    }

    #[test]
    fn max_nodes_truncates() {
        let mut tracker = Tracker::default();
        let (uids, removed) = pass(&mut tracker, &[0x10, 0x20, 0x30], 2);
        assert_eq!(uids, vec![Some(1), Some(2), None]);
        assert!(removed.is_empty());
        assert!(tracker.truncated);
        assert_eq!(tracker.emitted, 2);

        // A node pushed beyond the limit is removed.
        let (uids, removed) = pass(&mut tracker, &[0x30, 0x10, 0x20], 2);
        assert_eq!(uids, vec![Some(3), Some(1), None]);
        assert_eq!(removed, vec![2]);

        let (uids, removed) = pass(&mut tracker, &[0x10], 2);
        assert_eq!(uids, vec![Some(1)]);
        assert_eq!(removed, vec![3]);
        assert!(!tracker.truncated);
    }

    #[test]
    fn disabling_forgets_tracked_objects() {
        let mut mirror = Mirror::new();
        mirror.set_enabled(true);
        assert!(mirror.reset_pending);
        pass(&mut mirror.tracker, &[0x10, 0x20], 100);
        mirror.mark_structure();
        assert_eq!(mirror.tracker.nodes.len(), 2);

        mirror.set_enabled(false);
        assert!(mirror.tracker.nodes.is_empty());
        assert!(mirror.tracked_objects.is_empty());
        assert!(!mirror.structure_dirty && !mirror.reset_pending);

        // Hooks do nothing while disabled.
        mirror.mark_structure();
        assert!(!mirror.structure_dirty);

        // A filter change forgets too, and resets the receiver.
        mirror.set_enabled(true);
        pass(&mut mirror.tracker, &[0x10], 100);
        mirror.set_filter(MirrorFilter::default());
        assert!(mirror.tracker.nodes.is_empty());
        assert!(mirror.reset_pending);
    }

    #[test]
    fn filter_defaults() {
        let filter = MirrorFilter::default();
        assert!(filter.interactive && filter.text && filter.identified);
        assert!(!filter.emit_hidden);
        assert!(filter.exclude_classes.is_empty());
        assert_eq!(filter.max_nodes, 5000);
    }

    #[test]
    fn strings_are_truncated() {
        assert_eq!(truncate("abc".to_string()), "abc");
        let long: String = "é".repeat(MAX_STRING_LENGTH + 10);
        assert_eq!(truncate(long).chars().count(), MAX_STRING_LENGTH);
    }
}
