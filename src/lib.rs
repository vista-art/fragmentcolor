pub mod controllers;
mod events;
mod platform;
mod renderer;
mod shapes;

use cfg_if::cfg_if;
use controllers::ControllerOptions;
use events::{window::WindowOptions, EventManager};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, sync::RwLock};
#[cfg(wasm)]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::prelude::*};
// use enrichments::Enrichment; //@TODO

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    pub window: Option<WindowOptions>,
    pub controllers: Option<ControllerOptions>,
}

#[derive(Clone)]
#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
pub struct Vip {
    event_manager: Arc<RwLock<EventManager>>,
    //enrichment: Vec<Box<dyn Enrichment>>, //@TODO
}

#[cfg(not(wasm))]
unsafe impl Send for Vip {}

#[cfg_attr(wasm, wasm_bindgen)]
impl Vip {
    #[cfg_attr(wasm, wasm_bindgen(constructor))]
    pub fn new() -> Vip {
        Self::init_logger();

        Self {
            event_manager: Arc::new(RwLock::new(EventManager::new())),
        }
    }

    fn event_manager_write_lock(&self) -> std::sync::RwLockWriteGuard<'_, EventManager> {
        self.event_manager
            .write()
            .expect("Couldn't get event manager write lock")
    }

    fn event_manager(&self) -> std::sync::RwLockReadGuard<'_, EventManager> {
        self.event_manager
            .read()
            .expect("Couldn't get event manager read lock")
    }

    #[cfg(wasm)]
    pub fn config(&mut self, options: JsValue) {
        let options: Options = options.into_serde().expect("Couldn't deserialize options");

        let mut event_manager = self.event_manager_write_lock();
        let renderer = event_manager.config(options);

        let future = async move {
            renderer.borrow_mut().initialize().await;
        };

        wasm_bindgen_futures::spawn_local(future);
    }

    #[cfg(not(wasm))]
    pub async fn config(&mut self, options: Options) {
        let mut event_manager = self.event_manager_write_lock();
        let renderer = event_manager.config(options);
        renderer.borrow_mut().initialize().await;
    }

    pub fn run(&mut self) {
        let mut event_manager = self.event_manager_write_lock();
        let event_handler = event_manager.get_event_handler();
        drop(event_manager);

        #[cfg(wasm)]
        wasm_bindgen_futures::spawn_local(event_handler.run());

        #[cfg(not(wasm))]
        pollster::block_on(event_handler.run()); // this function never returns
    }

    // @TODO params should be dynamic and of arbitrary type
    pub fn trigger(&self, event: &str, param1: f32, param2: f32) {
        self.event_manager()
            .trigger(event, param1, param2)
            .expect("Event loop closed");
    }

    fn init_logger() {
        cfg_if! { if #[cfg(wasm)] {
            crate::platform::web::utils::set_panic_hook();
            console_log::init_with_level(log::Level::Info).unwrap_or(());
        } else {
            env_logger::try_init().unwrap_or(());
        }}
    }

    // @TODO this is a temporary debug function to inject
    // events in the renderer. This will be dynamic later.
    pub fn set_normalized_position(&self, x: f32, y: f32) {
        use log::info;
        info!("from set_normalized_position: x: {}, y: {}", &x, &y);
        self.trigger("gaze::set_normalized_position", x, y);
    }

    #[cfg(wasm)]
    pub fn update(_config: &[u8]) {
        let _ = gloo_utils::window().alert_with_message("Hello, update!");
    }

    #[cfg(wasm)]
    pub fn resize(_width: u32, _height: u32) {
        let _ = gloo_utils::window().alert_with_message("Hello, resize!");
    }

    #[cfg(wasm)]
    pub fn render() {
        // NOTE:
        // Decoding jpegs in WASM isn't performant, as it does not support threads.
        // If you want to speed up image loading in general in WASM you could
        // use the browser's built-in decoders instead of image when building
        // with wasm-bindgen. This will involve creating an <img> tag in Rust
        // to get the image, and then a <canvas> to get the pixel data.
        let _ = gloo_utils::window().alert_with_message("Hello, render!");
    }

    #[cfg(wasm)]
    pub fn clear() {
        let _ = gloo_utils::window().alert_with_message("Hello, clear!");
    }

    #[cfg(wasm)]
    pub fn hide() {
        let _ = gloo_utils::window().alert_with_message("Hello, hide!");
    }

    #[cfg(wasm)]
    pub fn show() {
        let _ = gloo_utils::window().alert_with_message("Hello, show!");
    }
}
