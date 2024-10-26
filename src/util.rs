use std::{io::Error, task::Waker, thread::LocalKey};

use js_sys::{JsString, Object, Reflect};
use wasm_bindgen::JsValue;

use crate::GETTING_JS_FIELD_ERROR;

pub(crate) fn get_value(target: &JsValue, key: &'static LocalKey<JsString>) -> JsValue {
    let key = key.with(JsString::clone);
    Reflect::get(target, &key).expect(&format!("{}, key: \"{}\"", GETTING_JS_FIELD_ERROR, key))
}
pub(crate) fn set_value(target: &JsValue, key: &'static LocalKey<JsString>, value: &JsValue) {
    Reflect::set(target, &key.with(JsString::clone), value)
        .expect("Setting js field error, this is an error of the crate.");
}
pub(crate) fn get_value_as_f64(target: &JsValue, key: &'static LocalKey<JsString>) -> f64 {
    get_value(target, key)
        .as_f64()
        .expect("Converting js field to f64 error, this is an error of the crate.")
}

pub(crate) struct Task<T> {
    pub(crate) waker: Option<Waker>,
    pub(crate) result: Option<T>,
}

pub(crate) fn js_value_to_string(v: JsValue) -> String {
    format!("{}", Object::from(v).to_string())
}
pub(crate) fn js_value_to_error(v: JsValue) -> Error {
    Error::other(js_value_to_string(v))
}
