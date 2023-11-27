use gloo_utils::document;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use winit::{
    event_loop::EventLoop,
    platform::web::WindowBuilderExtWebSys,
    window::{Window, WindowBuilder},
};

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CanvasOptions {
    canvas_selector: Option<String>,
    decorations: Option<bool>,
    fullscreen: Option<bool>,
    resizable: Option<bool>,
    title: Option<String>,
    size: Option<(u32, u32)>,
}

pub fn init_window<T>(event_loop: &EventLoop<T>, options: &CanvasOptions) -> Window {
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
