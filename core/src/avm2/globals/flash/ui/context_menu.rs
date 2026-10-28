use crate::avm2::activation::Activation;
use crate::avm2::globals::slots::flash_display_native_menu_item as native_item_slots;
use crate::avm2::globals::slots::flash_ui_context_menu as menu_slots;
use crate::avm2::globals::slots::flash_ui_context_menu_built_in_items as builtins_slots;
use crate::avm2::globals::slots::flash_ui_context_menu_clipboard_items as clipboard_slots;
use crate::avm2::globals::slots::flash_ui_context_menu_item as item_slots;
use crate::avm2::object::{Object, TObject as _};
use crate::avm2::value::Value;
use crate::context_menu;
use crate::display_object::{DisplayObject, TDisplayObject};

pub fn make_context_menu_state<'gc>(
    menu: Option<Object<'gc>>,
    object: Option<DisplayObject<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> context_menu::ContextMenuState<'gc> {
    let mut result = context_menu::ContextMenuState::new();

    result.set_display_object(object);

    macro_rules! check_bool {
        ( $obj:expr, $slot:expr, $value:expr ) => {
            matches!($obj.get_slot($slot), Value::Bool($value))
        };
    }

    let mut builtin_items = context_menu::BuiltInItemFlags::for_stage(activation.context.stage);
    if let Some(menu) = menu
        && let Some(builtins) = menu.get_slot(menu_slots::_BUILT_IN_ITEMS).as_object()
    {
        if check_bool!(builtins, builtins_slots::_ZOOM, false) {
            builtin_items.zoom = false;
        }
        if check_bool!(builtins, builtins_slots::_QUALITY, false) {
            builtin_items.quality = false;
        }
        if check_bool!(builtins, builtins_slots::_PLAY, false) {
            builtin_items.play = false;
        }
        if check_bool!(builtins, builtins_slots::_LOOP, false) {
            builtin_items.loop_ = false;
        }
        if check_bool!(builtins, builtins_slots::_REWIND, false) {
            builtin_items.rewind = false;
        }
        if check_bool!(builtins, builtins_slots::_FORWARD_AND_BACK, false) {
            builtin_items.forward_and_back = false;
        }
        if check_bool!(builtins, builtins_slots::_PRINT, false) {
            builtin_items.print = false;
        }
    }

    // When the clipboard menu is enabled, the clipboard items replace the
    // built-in items, and the clipboard events are sent to the menu owner.
    let clipboard_target = menu
        .filter(|menu| check_bool!(menu, menu_slots::_CLIPBOARD_MENU, true))
        .and(object)
        .and_then(|object| object.object2())
        .map(Object::from);
    if let Some(target) = clipboard_target {
        let mut flags = context_menu::ClipboardItemFlags::default();
        if let Some(items) =
            menu.and_then(|menu| menu.get_slot(menu_slots::_CLIPBOARD_ITEMS).as_object())
        {
            flags.cut = check_bool!(items, clipboard_slots::_CUT, true);
            flags.copy = check_bool!(items, clipboard_slots::_COPY, true);
            flags.paste = check_bool!(items, clipboard_slots::_PASTE, true);
            flags.clear = check_bool!(items, clipboard_slots::_CLEAR, true);
            flags.select_all = check_bool!(items, clipboard_slots::_SELECT_ALL, true);
        }
        result.build_avm2_clipboard_items(flags, target, activation.context);
    } else {
        result.build_builtin_items(builtin_items, activation.context);
    }

    if let Some(menu) = menu
        && let Value::Object(custom_items) = menu.get_slot(menu_slots::_CUSTOM_ITEMS)
        // note: this borrows the array, but it shouldn't be possible for
        // AS to get invoked here and cause BorrowMutError
        && let Some(array) = custom_items.as_array_storage()
    {
        let context_menu_item_class = activation.avm2().class_defs().contextmenuitem;

        for (i, item) in array.iter().enumerate() {
            // TODO: Non-CustomMenuItem Object-s shouldn't count

            if let Some(Value::Object(item)) = item
                && item.is_of_type(context_menu_item_class)
            {
                let caption = if let Value::String(s) = item.get_slot(item_slots::_CAPTION) {
                    s
                } else {
                    continue;
                };

                let enabled = check_bool!(item, native_item_slots::_ENABLED, true);
                let visible = check_bool!(item, item_slots::_VISIBLE, true);
                let separator_before = check_bool!(item, item_slots::_SEPARATOR_BEFORE, true);

                if !visible {
                    continue;
                }

                result.push(
                    context_menu::ContextMenuItem {
                        enabled,
                        separator_before: separator_before || i == 0,
                        caption: caption.to_string(),
                        checked: false,
                    },
                    context_menu::ContextMenuCallback::Avm2 { item },
                );
            }
        }
    }
    result
}
