#![cfg(wasm)]
#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod canvas;
mod scene;
mod shapes;

use gloo_utils::format::JsValueSerdeExt;
pub use plr::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = FragmentColor)]
pub struct JsFragmentColor;

#[wasm_bindgen(js_class = FragmentColor)]
impl JsFragmentColor {
    #[wasm_bindgen]
    pub fn config(options: JsValue) {
        let options: AppOptions = options.into_serde().unwrap();
        FragmentColor::config(options);
    }

    #[wasm_bindgen]
    pub fn run() {
        FragmentColor::run();
    }
}
