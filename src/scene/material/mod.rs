//! `Material`: what a `Model` draws its mesh with, one of the material types or a `Shader`.
//!
//! Each shading model has a type of its own, the way `Target` has `WindowTarget` and
//! `TextureTarget`, and offers only the settings its shader reads. `Material` plays the
//! part `RenderTarget` plays beside `Target`: `Model::new` takes any of them through
//! `From`, and `Model::material` and the glTF loader return it.

use lsp_doc::lsp_doc;

use crate::core::UniformInput;
use crate::{PbrMaterial, Shader, ShaderError};

mod alpha_mode;
pub use alpha_mode::AlphaMode;

mod error;
pub use error::*;

pub(crate) mod platform;

/// The light types and grid constants, which the light binning pass shares.
pub(crate) const LIGHT: &str = include_str!("light.wgsl");

/// What a `Model` draws its mesh with: one of the material types, or a `Shader` of your own.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[lsp_doc("docs/api/scene/material/material.md")]
pub enum Material {
    /// glTF's metallic-roughness model, on FragmentColor's built-in PBR shader.
    Pbr(PbrMaterial),
    /// A shader of your own. The renderer fills the `camera`, `lights` and `model`
    /// slots it declares, and it sets its other values with `Shader::set`.
    Custom(Shader),
}

impl Material {
    /// The shader the material draws with.
    pub(crate) fn shader(&self) -> &Shader {
        match self {
            Material::Pbr(material) => material.shader(),
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
