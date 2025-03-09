pub mod all;

#[cfg(wasm)]
pub mod web;

#[cfg(android)]
pub mod android;

#[cfg(ios)]
pub mod ios;

#[cfg(not(any(wasm, android, ios)))]
pub mod desktop;

// @TODO
// pub enum PlatformSurface {
//     AndroidNativeWindow(crate::platform::android::AndroidNativeWindow),
//     HtmlCanvas(web_sys::HtmlCanvasElement),
//     WinitWindow(winit::window::Window),
//     MetalLayer(metal_rs::CoreAnimationLayer),
// }
