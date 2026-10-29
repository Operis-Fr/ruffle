//! Conversions between JavaScript values and the AMF0 values carried by
//! `LocalConnection` messages, for the JavaScript `LocalConnection` API.

use js_sys::{Array, Date, Object, Reflect};
use ruffle_core::{AmfElement, AmfObjectId, AmfValue};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};

/// Converts an AMF value received by a JavaScript connection.
pub fn amf_to_js_value(value: &AmfValue) -> JsValue {
    match value {
        AmfValue::Number(number) => JsValue::from_f64(*number),
        AmfValue::Integer(integer) => JsValue::from_f64(f64::from(*integer)),
        AmfValue::Bool(boolean) => JsValue::from_bool(*boolean),
        AmfValue::String(string) | AmfValue::XML(string, _) => JsValue::from_str(string),
        AmfValue::Null => JsValue::NULL,
        AmfValue::Date(time, _) => Date::new(&JsValue::from_f64(*time)).into(),
        AmfValue::AMF3(inner) => amf_to_js_value(inner),
        AmfValue::StrictArray(_, values) => values
            .iter()
            .map(|value| amf_to_js_value(value))
            .collect::<Array>()
            .into(),
        AmfValue::ECMAArray(_, dense, sparse, _) => {
            let array: Array = dense.iter().map(|value| amf_to_js_value(value)).collect();
            set_elements(&array, sparse);
            array.into()
        }
        AmfValue::Object(_, elements, _) => {
            let object = Object::new();
            set_elements(&object, elements);
            object.into()
        }
        _ => JsValue::UNDEFINED,
    }
}

fn set_elements(target: &JsValue, elements: &[AmfElement]) {
    for element in elements {
        let _ = Reflect::set(
            target,
            &JsValue::from_str(&element.name),
            &amf_to_js_value(&element.value),
        );
    }
}

/// Converts a JavaScript value sent to a movie's connection, as `LocalConnection.send`
/// would serialize it (AMF0: arrays become ECMA arrays, plain objects anonymous objects).
pub fn js_to_amf_value(value: &JsValue) -> AmfValue {
    if let Some(boolean) = value.as_bool() {
        AmfValue::Bool(boolean)
    } else if let Some(number) = value.as_f64() {
        AmfValue::Number(number)
    } else if let Some(string) = value.as_string() {
        AmfValue::String(string)
    } else if value.is_null() {
        AmfValue::Null
    } else if Array::is_array(value) {
        let array: &Array = value.unchecked_ref();
        let dense: Vec<_> = array
            .iter()
            .map(|value| Rc::new(js_to_amf_value(&value)))
            .collect();
        let length = dense.len() as u32;
        AmfValue::ECMAArray(AmfObjectId::INVALID, dense, vec![], length)
    } else if let Some(date) = value.dyn_ref::<Date>() {
        AmfValue::Date(date.get_time(), None)
    } else if value.is_object() {
        let elements = Object::entries(value.unchecked_ref())
            .iter()
            .filter_map(|entry| {
                let entry: Array = entry.unchecked_into();
                let name = entry.get(0).as_string()?;
                Some(AmfElement {
                    name,
                    value: Rc::new(js_to_amf_value(&entry.get(1))),
                })
            })
            .collect();
        AmfValue::Object(AmfObjectId::INVALID, elements, None)
    } else {
        AmfValue::Undefined
    }
}
