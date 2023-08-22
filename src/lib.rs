pub mod enrichments;
mod events;
mod renderer;
mod state;

use std::{sync::Arc, sync::RwLock};

use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::prelude::*};

use serde::{Deserialize, Serialize};

use enrichments::{gaze::GazeEvent, Enrichments};
use events::{EventManager, VipEvent};
use winit::event_loop::EventLoopClosed;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    pub canvas_selector: Option<String>,
    pub enrichments: Enrichments,
}

#[derive(Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Vip {
    pub event_manager: Arc<RwLock<EventManager>>,
}

unsafe impl Send for Vip {}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Vip {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Vip {
        Self::init_logger();

        Self {
            event_manager: Arc::new(RwLock::new(EventManager::new())),
        }
    }

    fn event_manager(&self) -> std::sync::RwLockWriteGuard<'_, EventManager> {
        self.event_manager
            .write()
            .expect("Couldn't get event manager")
    }

    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn config_and_run(&self, options: JsValue) {
        let options: Options = options.into_serde().expect("Couldn't deserialize options");

        let mut event_manager = self.event_manager.borrow_mut();
        let event_loop = event_manager.event_loop.take().expect("Event loop not set");
        let canvas_selector = options
            .canvas_selector
            .as_ref()
            .expect("Canvas selector not set");
        let window = events::init_window(&event_loop, canvas_selector);

        let state = state::State::new(window, options.enrichments).await;

        wasm_bindgen_futures::spawn_local(events::run_event_loop(event_loop, state));
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn config(&mut self, options: Options) {
        let mut event_manager = self.event_manager.write().unwrap();
        event_manager.config(options).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&mut self) {
        let mut event_manager = self.event_manager.write().unwrap();
        let event_loop_container = *event_manager.event_loop_container.take().unwrap();

        // release to avoid deadlock
        drop(event_manager);

        event_loop_container.run();
    }

    pub fn trigger(&self, event: events::VipEvent) -> Result<(), EventLoopClosed<VipEvent>> {
        self.event_manager().trigger(event)
    }

    fn init_logger() {
        cfg_if! { if #[cfg(target_arch = "wasm32")] {
            utils::set_panic_hook();
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }}
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn set_position(&self, x: u32, y: u32) -> Result<(), EventLoopClosed<VipEvent>> {
        self.event_manager
            .read()
            .expect("Couldn't get event manager")
            .event_loop_proxy
            .read()
            .unwrap()
            .send_event(VipEvent::Gaze(GazeEvent::ChangePosition { x, y }))

        //self.trigger(VipEvent::Gaze(GazeEvent::ChangePosition { x, y }))
    }

    pub fn set_normalized_position(&self, x: f32, y: f32) {
        println!("setting event manager for x: {}, y: {}", &x, &y);
        let event_manager_rwlock = &self.event_manager;

        println!("read locking event manager");
        let result = event_manager_rwlock.read();

        println!("event manager locked. getting event manager");
        let event_manager = result.expect("Couldn't get event manager");

        println!("got event manager. locking event loop proxy");
        let event_manager_proxy = event_manager.event_loop_proxy.read().unwrap();

        println!("got event loop proxy. sending event");
        event_manager_proxy
            .send_event(VipEvent::Gaze(GazeEvent::ChangeNormalizedPosition { x, y }))
            .ok();

        println!("sent event");

        //self.trigger(VipEvent::Gaze(GazeEvent::ChangeNormalizedPosition { x, y }))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update(_config: &[u8]) {
        let _ = gloo_utils::window().alert_with_message("Hello, update!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn resize(_width: u32, _height: u32) {
        let _ = gloo_utils::window().alert_with_message("Hello, resize!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn render() {
        // NOTE:
        // Decoding jpegs in WASM isn't performant, as it does not support threads.
        // If you want to speed up image loading in general in WASM you could
        // use the browser's built-in decoders instead of image when building
        // with wasm-bindgen. This will involve creating an <img> tag in Rust
        // to get the image, and then a <canvas> to get the pixel data.
        let _ = gloo_utils::window().alert_with_message("Hello, render!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn clear() {
        let _ = gloo_utils::window().alert_with_message("Hello, clear!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn hide() {
        let _ = gloo_utils::window().alert_with_message("Hello, hide!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn show() {
        let _ = gloo_utils::window().alert_with_message("Hello, show!");
    }
}
