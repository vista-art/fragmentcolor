//! Scene module — higher-level abstractions on top of Mesh + Material.
//!
//! Houses [`Scene`] (the top-level container), [`Model`] (Mesh + Material +
//! transform), [`Camera`], the unified [`Light`] type with three
//! constructors (`Light::directional` / `Light::point` / `Light::spot`),
//! and the [`SceneObject`] trait that ties them together. The split
//! mirrors glTF / USD: a Scene is a flat list of nodes (geometry,
//! viewpoints, lights), and the renderer walks the scene to produce a
//! frame.

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
pub(crate) use light::{GpuLight, LightObject, PassLights, bump_epoch, current_epoch};
pub use loader::{GltfSource, SceneLoadError, SceneSource};
pub use material::*;
pub use model::*;
pub use scene::{PassRef, Scene};
pub use scene_object::SceneObject;
