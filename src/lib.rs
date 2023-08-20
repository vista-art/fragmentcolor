mod color;
pub mod enrichments;
mod events;
mod screen;
mod state;
mod utils;
mod vertex;

use std::cell::RefCell;

use cfg_if::cfg_if;

#[cfg(feature = "camera")]
mod camera;
#[cfg(feature = "instances")]
mod instances;
#[cfg(feature = "texture")]
mod texture;

#[cfg(target_arch = "wasm32")]
use {
    gloo_utils::format::JsValueSerdeExt,
    js_sys::Promise,
    wasm_bindgen::prelude::*,
    wasm_bindgen_futures::{future_to_promise, JsFuture},
};

use log::info;
use serde::{Deserialize, Serialize};

use enrichments::Enrichments;
use events::EventManager;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    pub canvas_selector: Option<String>,
    pub enrichments: Enrichments,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Vip {
    event_manager: RefCell<EventManager>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Vip {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Vip {
        Self::init_logger();

        Self {
            event_manager: RefCell::new(EventManager::new()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn config_and_run(&self, options: JsValue) {
        let options: Options = options.into_serde().unwrap();

        let mut event_manager = self.event_manager.borrow_mut();
        let event_loop = event_manager.event_loop.take().expect("Event loop not set");
        let window = events::init_window(&event_loop, options.canvas_selector.as_ref().unwrap());

        wasm_bindgen_futures::spawn_local(async {
            let state = state::State::new(window, options.enrichments).await;
            events::run_event_loop(event_loop, state)
        });

        // let promise = Promise::new(&mut |resolve, reject| {

        //     let returns_future_result = || async { Result::<_, ()>::Ok(()) };

        //     wasm_bindgen_futures::spawn_local(async move {
        //         match returns_future_result.await {
        //             Ok(val) => {
        //                 resolve.call1(&JsValue::undefined(), &val).unwrap_throw();
        //             }
        //             Err(val) => {
        //                 reject.call1(&JsValue::undefined(), &val).unwrap_throw();
        //             }
        //         }
        //     });
        // });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn config(&self, options: Options) {
        let mut event_manager = self.event_manager.borrow_mut();
        event_manager.create_state(options).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&self) {
        pollster::block_on((self.event_manager.borrow_mut()).run());
    }

    fn init_logger() {
        cfg_if! { if #[cfg(target_arch = "wasm32")] {
            utils::set_panic_hook();
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }}
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update(_config: &[u8]) {
        let _ = gloo_utils::window().alert_with_message("Hello, update!");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn resize(_width: u32, _height: u32) {
        let _ = gloo_utils::window().alert_with_message("Hello, resize!");
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn set_position(x: u32, y: u32) {
        let message = format!("Position: {}, {}", x, y);
        info!("{}", message);
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init() -> Vip {
    Vip::new()
}
