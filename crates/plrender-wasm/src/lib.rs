#![cfg(wasm)]
#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod canvas;
mod scene;
mod shapes;

use gloo_utils::format::JsValueSerdeExt;
pub use plr::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PLRender)]
pub struct JsPLRender;

#[wasm_bindgen(js_class = PLRender)]
impl JsPLRender {
    #[wasm_bindgen]
    pub fn config(options: JsValue) {
        let options: AppOptions = options.into_serde().unwrap();
        PLRender::config(options);
    }

    #[wasm_bindgen]
    pub fn run() {
        PLRender::run();
    }
}
