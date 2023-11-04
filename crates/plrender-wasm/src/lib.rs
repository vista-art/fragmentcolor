#![cfg(wasm)]

#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod utils;
mod window;
use plrender::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PLRender, getter_with_clone)]
pub struct PLRenderWasm {
    plrender: Scene,
}

#[wasm_bindgen(js_class = PLRender)]
impl PLRenderWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Self {
        // @TODO Don't forget to inject "webgl2" as the power_preference
        //       in the App's RenderOptions struct.

        Self {
            plrender: Scene::new(options.into_serde().unwrap()),
        }
    }

    fn init_logger(level: Option<log::Level>) {
        console_error_panic_hook::set_once();
        console_log::init_with_level(level).unwrap_or(());
    }
}
