use std::{
    io::{Error, ErrorKind},
    task::Waker,
};

use js_sys::{JsString, Object, Reflect};
use wasm_bindgen::{JsCast, JsThreadLocal, JsValue};
use web_sys::DomException;

use crate::GETTING_JS_FIELD_ERROR;

pub(crate) fn get_value(target: &JsValue, key: &'static JsThreadLocal<JsString>) -> JsValue {
    let key = key.with(JsString::clone);
    Reflect::get(target, &key).expect(&format!("{}, key: \"{}\"", GETTING_JS_FIELD_ERROR, key))
}
pub(crate) fn set_value(target: &JsValue, key: &'static JsThreadLocal<JsString>, value: &JsValue) {
    Reflect::set(target, &key.with(JsString::clone), value)
        .expect("Setting js field error, this is an error of the crate.");
}
pub(crate) fn get_value_as_f64(target: &JsValue, key: &'static JsThreadLocal<JsString>) -> f64 {
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
    if let Ok(e) = v.clone().dyn_into::<DomException>() {
        if e.name() == "NotFoundError" {
            Error::from(ErrorKind::NotFound)
        } else if e.name() == "NotAllowedError" {
            Error::from(ErrorKind::PermissionDenied)
        } else if e.name() == "NoModificationAllowedError" {
            Error::from(ErrorKind::PermissionDenied)
        } else {
            Error::other(js_value_to_string(v))
        }
    } else {
        Error::other(js_value_to_string(v))
    }
}
