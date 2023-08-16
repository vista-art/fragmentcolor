mod enrichments;
mod events;
mod state;
mod utils;

use cfg_if::cfg_if;
use enrichments::gaze::GazeConfig;
use state::State;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use {
    gloo_utils::{document, format::JsValueSerdeExt},
    wasm_bindgen::JsValue,
    winit::platform::web::WindowBuilderExtWebSys,
};

pub struct Options {
    canvas_selector: Option<String>,
    gaze: Option<GazeConfig>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            canvas_selector: None,
            gaze: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub async fn run(options: Options) {
    init_logger();

    let event_loop = EventLoop::new();

    cfg_if! { if #[cfg(target_arch = "wasm32")] {
        let window = init_canvas_window(&event_loop, &options);
    } else {
        let window = init_window(&event_loop);
    }}

    if options.canvas_selector.is_none() {}
    if options.gaze.is_some() {
        let GazeConfig {
            size: _size,
            color: _color,
            opacity: _opacity,
        } = options.gaze.expect("Couldn't get gaze config");
    }

    let state = State::new(window).await;

    events::run_event_loop(event_loop, state);
}

fn init_logger() {
    cfg_if! { if #[cfg(target_arch = "wasm32")] {
        utils::set_panic_hook();
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    } else {
        env_logger::init();
    }}
}

fn init_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
        .build(event_loop)
        .expect("Couldn't build window");

    window
}

#[cfg(target_arch = "wasm32")]
fn init_canvas_window(event_loop: &EventLoop<()>, options: &Options) -> Window {
    let canvas_selector = options
        .canvas_selector
        .as_ref()
        .expect("Invalid canvas selector");

    let canvas: Option<web_sys::HtmlCanvasElement> = document()
        .query_selector(canvas_selector)
        .unwrap()
        .expect("Couldn't get canvas")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok();

    let size = canvas
        .as_ref()
        .expect("Couldn't get canvas size")
        .get_bounding_client_rect();

    let window = WindowBuilder::new()
        .with_canvas(canvas)
        .build(event_loop)
        .expect("Couldn't build canvas context");

    window.set_inner_size(winit::dpi::LogicalSize::new(
        size.width() as u32,
        size.height() as u32,
    ));

    window
}

cfg_if! { if #[cfg(target_arch = "wasm32")] {

    #[wasm_bindgen]
    pub fn greet(name: &str) {
        let _ = gloo_utils::window().alert_with_message(&format!("Hello, {}!", name));
    }

    #[wasm_bindgen]
    pub async fn config(canvas_selector: &str, config: JsValue) {
        let gaze = config.into_serde::<GazeConfig>().ok();

        wasm_bindgen_futures::spawn_local(run(Options {
            canvas_selector: Some(canvas_selector.to_string()),
            gaze,
        }));
    }

    #[wasm_bindgen]
    pub fn update(_config: &[u8]) {
        let _ = gloo_utils::window().alert_with_message("Hello, update!");
    }

    #[wasm_bindgen]
    pub fn resize(_width: u32, _height: u32) {
        let _ = gloo_utils::window().alert_with_message("Hello, resize!");
    }

    #[wasm_bindgen]
    pub fn position(_x: u32, _y: u32) {
        let _ = gloo_utils::window().alert_with_message("Hello, position!");
    }

    #[wasm_bindgen]
    pub fn render() {
        let _ = gloo_utils::window().alert_with_message("Hello, render!");
    }

    #[wasm_bindgen]
    pub fn clear() {
        let _ = gloo_utils::window().alert_with_message("Hello, clear!");
    }

    #[wasm_bindgen]
    pub fn hide() {
        let _ = gloo_utils::window().alert_with_message("Hello, hide!");
    }

    #[wasm_bindgen]
    pub fn show() {
        let _ = gloo_utils::window().alert_with_message("Hello, show!");
    }

} else {
}}
