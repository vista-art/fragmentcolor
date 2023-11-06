#![cfg(wasm)]

#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod utils;
mod window;

pub use plr::app::window::{Window, WindowOptions};
use plr::{App, AppOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = App, getter_with_clone)]
pub struct WasmApp {
    inner: App,
}

#[wasm_bindgen(js_class = App)]
impl WasmApp {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Self {
        Self {
            inner: App::new(AppOptions {
                device_limits: "webgl2",
                ..default::Default()
            }),
        }
    }

    fn init_logger(level: Option<log::Level>) {
        console_error_panic_hook::set_once();
        console_log::init_with_level(level).unwrap_or(());
    }
}

pub struct WasmWindow {
    inner: Window,
}

#[wasm_bindgen(js_class = Canvas)]
impl WasmWindow {
    #[wasm_bindgen(constructor)]
    pub fn new(app: &WasmApp) -> Self {
        Self {
            inner: window::init_window(app.inner.event_loop().window_target()),
        }
    }

    // pub struct WindowOptions {
    //     pub decorations: Option<bool>,
    //     pub fullscreen: Option<bool>,
    //     pub resizable: Option<bool>,
    //     pub title: Option<String>,
    //     pub size: Option<(u32, u32)>,
    //     pub min_size: Option<(u32, u32)>,
    //     pub max_size: Option<(u32, u32)>,
    // }
}
