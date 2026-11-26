//! `Scene`, the flat list of models, cameras and lights a frame shows, as in glTF and USD,
//! with `Model`, `Camera`, `Light`, the material types, the glTF loader and the `SceneObject` trait.

mod camera;
mod finite;
mod light;
mod loader;
mod material;
mod material_map;
mod model;
mod pbr_material;
mod platform;
#[allow(clippy::module_inception)]
mod scene;
mod scene_object;
mod unlit_material;

pub use camera::*;
pub(crate) use finite::{NotFinite, check_finite};
pub use light::*;
pub(crate) use light::{GpuLight, LightObject, PassLights, bump_epoch, read_epoch};
pub use loader::{GltfSource, SceneLoadError, SceneSource};
#[cfg(wasm)]
pub(crate) use material::platform::web::read_alpha_mode;
pub use material::*;
pub use material_map::MaterialMap;
pub(crate) use material_map::{MapSlot, UvSet, UvTransform};
pub use model::*;
pub use pbr_material::PbrMaterial;
pub use scene::{PassRef, Scene};
pub use scene_object::SceneObject;
pub use unlit_material::UnlitMaterial;
pub(crate) use scene_object::push_uniform;
