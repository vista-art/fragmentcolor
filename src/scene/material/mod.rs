//! `Material`: what a `Model` draws its mesh with, one of the material types or a `Shader`.
//!
//! Each shading model has a type of its own, the way `Target` has `WindowTarget` and
//! `TextureTarget`, and offers only the settings its shader reads. `Material` plays the
//! part `RenderTarget` plays beside `Target`: `Model::new` takes any of them through
//! `From`, and `Model::material` and the glTF loader return it.

use lsp_doc::lsp_doc;

use crate::core::UniformInput;
use crate::{PbrMaterial, Shader, ShaderError, UnlitMaterial};

mod alpha_mode;
pub use alpha_mode::AlphaMode;

mod error;
pub use error::*;

pub(crate) mod platform;

/// The light types and grid constants, which the light binning pass shares.
pub(crate) const LIGHT: &str = include_str!("light.wgsl");

/// The registry snippets the built-in material shaders compose from, baked in
/// directly rather than resolved through the shader registry. They power the
/// engine's own `PbrMaterial::new()` and `UnlitMaterial::new()`, so they must be
/// present on every platform regardless of which `shaders-<category>` features the
/// consumer enabled, including the slim web build (`--no-default-features`), whose
/// synchronous constructors cannot fetch them over the network. Embedding the files
/// (a few KB) keeps both constructors infallible and offline everywhere without
/// dragging the rest of the registry into the wasm bundle.
pub(crate) const MESH_TRANSFORM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/mesh/transform.wgsl"
));
pub(crate) const LINEAR_TO_SRGB: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/color/linear_to_srgb.wgsl"
));

/// What a `Model` draws its mesh with: one of the material types, or a `Shader` of your own.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[lsp_doc("docs/api/scene/material/material.md")]
pub enum Material {
    /// glTF's metallic-roughness model, on FragmentColor's built-in PBR shader.
    Pbr(PbrMaterial),
    /// A base color times its map, with no lighting, as glTF's `KHR_materials_unlit`.
    Unlit(UnlitMaterial),
    /// A shader of your own. The renderer fills the `camera`, `lights` and `model`
    /// slots it declares, and it sets its other values with `Shader::set`.
    Custom(Shader),
}

impl Material {
    /// The shader the material draws with.
    pub(crate) fn shader(&self) -> &Shader {
        match self {
            Material::Pbr(material) => material.shader(),
            Material::Unlit(material) => material.shader(),
            Material::Custom(shader) => shader,
        }
    }
}

impl From<&Material> for Material {
    fn from(material: &Material) -> Self {
        material.clone()
    }
}

impl From<PbrMaterial> for Material {
    fn from(material: PbrMaterial) -> Self {
        Material::Pbr(material)
    }
}

impl From<&PbrMaterial> for Material {
    fn from(material: &PbrMaterial) -> Self {
        Material::Pbr(material.clone())
    }
}

impl From<UnlitMaterial> for Material {
    fn from(material: UnlitMaterial) -> Self {
        Material::Unlit(material)
    }
}

impl From<&UnlitMaterial> for Material {
    fn from(material: &UnlitMaterial) -> Self {
        Material::Unlit(material.clone())
    }
}

impl From<Shader> for Material {
    fn from(shader: Shader) -> Self {
        Material::Custom(shader)
    }
}

impl From<&Shader> for Material {
    fn from(shader: &Shader) -> Self {
        Material::Custom(shader.clone())
    }
}

/// Writes each value under its key into `shader`, or none of them: every value is checked
/// against the shader before the first write. A key the shader doesn't declare is skipped
/// with a warning.
pub(crate) fn write_values(
    shader: &Shader,
    values: Vec<(String, UniformInput)>,
) -> Result<(), MaterialError> {
    let object = &shader.object;
    let mut checked = Vec::with_capacity(values.len());
    for (key, input) in values {
        match object.check_input(&key, &input) {
            Ok(()) => checked.push((key, input)),
            Err(error) if error.is_missing() => warn_skip(&key, &error),
            Err(error) => return Err(error.into()),
        }
    }
    for (key, input) in checked {
        object.set_input(&key, input)?;
    }
    Ok(())
}

/// Logs a key the material's shader doesn't declare, which a setter skips.
fn warn_skip(key: &str, error: &ShaderError) {
    log::warn!("Material setter '{key}' did not apply: {error}");
}

/// A write to `key`, where a key the shader doesn't declare is skipped with a warning.
pub(crate) fn skip_missing(
    key: &str,
    written: Result<(), ShaderError>,
) -> Result<(), MaterialError> {
    match written {
        Err(error) if error.is_missing() => {
            warn_skip(key, &error);
            Ok(())
        }
        written => Ok(written?),
    }
}
