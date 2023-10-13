#![cfg(wasm)]

#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod utils;
use plrender::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PLRender, getter_with_clone)]
pub struct PLRenderWasm {
    plrender: PLRender,
}

#[wasm_bindgen(js_class = PLRender)]
impl PLRenderWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Self {
        let options: Options = options.into_serde().unwrap();
        Self {
            plrender: PLRender::new(options),
        }
    }

    pub fn scene(&mut self, scene: JsValue) {
        let scene: Scene = scene.into_serde().unwrap();
        self.plrender.scene(scene);
    }

    pub fn update(&mut self) {
        self.plrender.update();
    }

    fn init_logger(level: Option<log::Level>) {
        console_error_panic_hook::set_once();
        console_log::init_with_level(level).unwrap_or(());
    }
}