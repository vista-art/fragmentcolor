//! Declares the targets and re-exports them.

mod target;
pub use target::*;

mod texture_target;
pub use texture_target::*;

mod window_target;
pub use window_target::*;

mod headless_window;
pub use headless_window::*;

mod render_target;
pub use render_target::*;

mod platform;
#[cfg(any(python, wasm, mobile))]
pub use platform::*;

mod error;
pub use error::*;
