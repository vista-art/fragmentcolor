#[cfg(wasm)]
use {
    gloo_utils::document, wasm_bindgen::prelude::*, winit::platform::web::WindowBuilderExtWebSys,
};

use serde::{Deserialize, Serialize};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WindowOptions {
    canvas_selector: Option<String>,
    _title: Option<String>,
}

#[cfg(not(wasm))]
pub fn init_window<T>(event_loop: &EventLoop<T>, _: &WindowOptions) -> Window {
    let window = WindowBuilder::new()
        .build(event_loop)
        .expect("Couldn't build window");

    window
}

#[cfg(wasm)]
pub fn init_window<T>(event_loop: &EventLoop<T>, options: &WindowOptions) -> Window {
    let canvas_selector = options
        .canvas_selector
        .as_ref()
        .expect("Canvas selector not set");

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
        .with_transparent(true)
        .build(event_loop)
        .expect("Couldn't build canvas context");

    window.set_inner_size(winit::dpi::LogicalSize::new(
        size.width() as u32,
        size.height() as u32,
    ));

    window
}
