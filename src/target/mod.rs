mod texture;
pub use texture::*;

mod window;
pub use window::*;

mod headless;
pub use headless::*;

mod all;
pub use all::*;

mod platform;
#[cfg(any(python, wasm))]
pub use platform::*;

use crate::size::Size;
use lsp_doc::lsp_doc;

pub mod error;

#[lsp_doc("docs/api/targets/target/target.md")]
pub trait Target {
    fn size(&self) -> Size;

    fn resize(&mut self, size: impl Into<Size>);

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError>;

    fn get_image(&self) -> Vec<u8>;
}

pub trait TargetFrame {
    fn view(&self) -> &wgpu::TextureView;
    fn format(&self) -> wgpu::TextureFormat;
    fn present(self: Box<Self>);
    fn auto_present(&self) -> bool {
        true
    }
}
