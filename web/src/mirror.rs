//! Web side of the display tree mirror: serialization of the frames and
//! conversion of injected input events.

use ruffle_core::PlayerEvent;
use ruffle_core::events::{
    KeyDescriptor, KeyLocation, LogicalKey, MouseButton, MouseWheelDelta, NamedKey, PhysicalKey,
    TextControlCode,
};
use ruffle_core::mirror::MirrorFrame;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

/// Serializes a frame as a plain object, with `null` for missing values.
pub fn mirror_frame_to_js(frame: &MirrorFrame) -> JsValue {
    frame
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::UNDEFINED)
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InjectedButton {
    #[default]
    Left,
    Right,
    Middle,
}

impl From<InjectedButton> for MouseButton {
    fn from(button: InjectedButton) -> Self {
        match button {
            InjectedButton::Left => MouseButton::Left,
            InjectedButton::Right => MouseButton::Right,
            InjectedButton::Middle => MouseButton::Middle,
        }
    }
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum InjectedTextControl {
    SelectAll,
    Backspace,
    Delete,
    Enter,
    Tab,
}

/// An input event sent by the embedder, in CSS pixels relative to the canvas.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MirrorInputEvent {
    MouseMove {
        x: f64,
        y: f64,
    },
    MouseDown {
        x: f64,
        y: f64,
        #[serde(default)]
        button: InjectedButton,
    },
    MouseUp {
        x: f64,
        y: f64,
        #[serde(default)]
        button: InjectedButton,
    },
    Wheel {
        x: f64,
        y: f64,
        #[serde(rename = "deltaY")]
        delta_y: f64,
    },
    Text {
        text: String,
    },
    TextControl {
        code: InjectedTextControl,
    },
}

impl MirrorInputEvent {
    /// The player events equivalent to this event, `device_pixel_ratio`
    /// converting CSS pixels to viewport pixels as the DOM listeners do.
    pub fn into_player_events(self, device_pixel_ratio: f64) -> Vec<PlayerEvent> {
        let mouse_move = |x: f64, y: f64| PlayerEvent::MouseMove {
            x: x * device_pixel_ratio,
            y: y * device_pixel_ratio,
        };
        match self {
            Self::MouseMove { x, y } => vec![mouse_move(x, y)],
            Self::MouseDown { x, y, button } => vec![
                mouse_move(x, y),
                PlayerEvent::MouseDown {
                    x: x * device_pixel_ratio,
                    y: y * device_pixel_ratio,
                    button: button.into(),
                    index: None,
                },
            ],
            Self::MouseUp { x, y, button } => vec![PlayerEvent::MouseUp {
                x: x * device_pixel_ratio,
                y: y * device_pixel_ratio,
                button: button.into(),
            }],
            Self::Wheel { x, y, delta_y } => vec![
                mouse_move(x, y),
                PlayerEvent::MouseWheel {
                    delta: MouseWheelDelta::Pixels(-delta_y),
                },
            ],
            Self::Text { text } => text
                .chars()
                .map(|codepoint| PlayerEvent::TextInput { codepoint })
                .collect(),
            Self::TextControl { code } => {
                let code = match code {
                    InjectedTextControl::SelectAll => TextControlCode::SelectAll,
                    InjectedTextControl::Backspace => TextControlCode::Backspace,
                    InjectedTextControl::Delete => TextControlCode::Delete,
                    InjectedTextControl::Enter => TextControlCode::Enter,
                    InjectedTextControl::Tab => {
                        // Tab is not a text control: it moves the focus.
                        let key = KeyDescriptor {
                            physical_key: PhysicalKey::Tab,
                            logical_key: LogicalKey::Named(NamedKey::Tab),
                            key_location: KeyLocation::Standard,
                        };
                        return vec![PlayerEvent::KeyDown { key }, PlayerEvent::KeyUp { key }];
                    }
                };
                vec![PlayerEvent::TextControl { code }]
            }
        }
    }
}
