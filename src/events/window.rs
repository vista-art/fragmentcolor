#[cfg(target_arch = "wasm32")]
use {
    gloo_utils::{document, window as web_window},
    wasm_bindgen::prelude::*,
    winit::platform::web::WindowBuilderExtWebSys,
};

use serde::{Deserialize, Serialize};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WindowOptions {
    canvas_selector: Option<String>,
    _title: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_window<T>(event_loop: &EventLoop<T>, _: &WindowOptions) -> Window {
    let window = WindowBuilder::new()
        .build(event_loop)
        .expect("Couldn't build window");

    window
}

#[cfg(target_arch = "wasm32")]
pub fn init_window<T>(event_loop: &EventLoop<T>, options: &WindowOptions) -> Window {
    let canvas_selector = options
        .canvas_selector
        .as_ref()
        .expect("Canvas selector not set");

    // This won't be needed in the near future.
    // See Winit EventLoop 3.0 Changes: https://github.com/rust-windowing/winit/issues/2900
    // --@TODO keep track of the upstream changes and remove this hack--
    //
    // Actually better @TODO: Remove this now. To do that, you need to separate the event_loop_runner
    // function from the event_handler callback. You then can inject a platform-specific runner.
    // For web, it would call event_loop.spawn(); for all other platforms, we'd stillcall run().
    // This change must happen in runner.rs.
    let _ = web_window().add_event_listener_with_callback(
        "unhandledrejection",
        &js_sys::Function::new_with_args(
            "event",
            "
                const message = event.reason.message;
                if (message.startsWith('Using exceptions for control flow')) {
                    event.preventDefault();
                }
            ",
        ),
    );

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
