use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[rustfmt::skip]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static OPEN: JsString = "Open";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static READ: JsString = "Read";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static WRITE: JsString = "Write";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static CLOSE: JsString = "Close";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static FLUSH: JsString = "Flush";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static TRUNCATE: JsString = "Truncate";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static DROP: JsString = "Drop";

    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static INDEX: JsString = "index";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static FD: JsString = "fd";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static BUF: JsString = "buf";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static SIZE: JsString = "size";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static ERROR: JsString = "error";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static OPTIONS: JsString = "options";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static HANDLE: JsString = "handle";
    #[wasm_bindgen(thread_local_v2, static_string)]
    pub static CURSOR: JsString = "cursor";
}
