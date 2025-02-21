#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, static_string)]
    static OPEN: JsString = "Open";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static READ: JsString = "Read";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static WRITE: JsString = "Write";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static CLOSE: JsString = "Close";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static FLUSH: JsString = "Flush";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static TRUNCATE: JsString = "Truncate";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static DROP: JsString = "Drop";

    #[wasm_bindgen(thread_local_v2, static_string)]
    static INDEX: JsString = "index";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static FD: JsString = "fd";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static BUF: JsString = "buf";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static SIZE: JsString = "size";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static ERROR: JsString = "error";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static OPTIONS: JsString = "options";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static HANDLE: JsString = "handle";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static CURSOR: JsString = "cursor";
}