use wasm_bindgen::prelude::*;

use web_sys::console;

#[wasm_bindgen]
pub fn hello_world() {
    console_error_panic_hook::set_once();
    console::error_1(&JsValue::from_str("Helo from WASM!"));
}
