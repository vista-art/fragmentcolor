pub(crate) mod buffer;
pub mod mesh;
pub mod resources;
pub(crate) mod sampler;
pub mod texture;

pub use resources::*;
pub use texture::*;

pub use mesh::*;

pub(crate) const IMAGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/images");
pub(crate) const SHADERS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/shaders");
