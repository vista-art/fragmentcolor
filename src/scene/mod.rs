//! `Scene`, the flat list of models, cameras and lights a frame shows, as in glTF and USD,
//! with `Model`, `Camera`, `Light`, `Material`, the glTF loader and the `SceneObject` trait.

mod camera;
mod light;
mod loader;
mod material;
mod model;
mod platform;
#[allow(clippy::module_inception)]
mod scene;
mod scene_object;

pub use camera::*;
pub use light::*;
pub(crate) use light::{GpuLight, LightObject, PassLights, bump_epoch, read_epoch};
pub use loader::{GltfSource, SceneLoadError, SceneSource};
pub use material::*;
pub use model::*;
pub use scene::{PassRef, Scene};
pub use scene_object::SceneObject;
