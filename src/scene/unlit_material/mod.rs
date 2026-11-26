//! `UnlitMaterial`: a base color times its map, with no lighting, as glTF's `KHR_materials_unlit`.
//!
//! It draws UI, sprites, models whose lighting is baked into their textures, and debug views.
//! It reads the `camera` root and no lights, and it takes the same vertex layout as
//! `PbrMaterial`: a `Vertex::pbr` mesh, or a mesh the glTF loader built. Its one map,
//! `base_color_map()`, starts unbound, and the renderer fills it with a white 1×1 fallback
//! until the caller sets one, so the material draws `base_color` alone.
//!
//! `UnlitMaterial` is `Clone` (shallow, Arc-share): cloning gives another handle to the same
//! shader state, as with `PbrMaterial`.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::core::UniformInput;
use crate::scene::material::{LINEAR_TO_SRGB, MESH_TRANSFORM, write_values};
use crate::scene::{MapSlot, UvSet, UvTransform, check_finite};
use crate::{AlphaMode, MaterialError, MaterialMap, Shader};

mod platform;

/// The main body of the unlit shader, which composes the `mesh/transform` and
/// `color/linear_to_srgb` snippets that `PbrMaterial` composes too.
const UNLIT_MAIN: &str = include_str!("unlit_main.wgsl");

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/unlit_material/unlit_material.md")]
pub struct UnlitMaterial {
    pub(crate) shader: Shader,
}

crate::impl_fc_kind!(UnlitMaterial, "UnlitMaterial");

impl Default for UnlitMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl UnlitMaterial {
    /// Build a white, opaque, single-sided unlit material.
    ///
    /// Infallible on every supported build configuration, as `PbrMaterial::new` is: the
    /// snippets its shader composes from are baked into the binary.
    #[lsp_doc("docs/api/scene/unlit_material/new.md")]
    pub fn new() -> Self {
        // SAFETY-justified expect: the parts are WGSL source compiled into the binary at build
        // time. The only way this fails is a malformed shader file in the repo, which is a
        // build-time programmer error caught by the tests, not a runtime fault.
        let shader = Shader::new([MESH_TRANSFORM, LINEAR_TO_SRGB, UNLIT_MAIN])
            .expect("SAFETY: built-in unlit shader parts are compiled-in WGSL source; a parse failure is a build-time error, not a runtime fault");
        let material = Self { shader };
        material.apply_defaults();
        material.shader.object.set_alpha_mode(AlphaMode::default());
        material
    }

    #[lsp_doc("docs/api/scene/unlit_material/shader.md")]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    fn apply_defaults(&self) {
        let _ = self
            .shader
            .set("material.base_color", [1.0_f32, 1.0, 1.0, 1.0]);
        let _ = self.shader.set("material.alpha_cutoff", 0.5_f32);
        let _ = self.shader.set("material.alpha_mode_flag", 0_u32);
        let _ = self
            .base_color_map()
            .set_uv(UvSet::First, UvTransform::default());
        let identity = [
            [1.0_f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let _ = self.shader.set("camera.view_proj", identity);
        let _ = self.shader.set("camera.position", [0.0_f32, 0.0, 0.0]);
    }

    #[lsp_doc("docs/api/scene/unlit_material/base_color.md")]
    pub fn base_color(&self, color: [f32; 4]) -> Result<Self, MaterialError> {
        self.set_factor("UnlitMaterial::base_color", "material.base_color", &color)
    }

    #[lsp_doc("docs/api/scene/unlit_material/alpha_cutoff.md")]
    pub fn alpha_cutoff(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.alpha_cutoff";
        self.set_factor("UnlitMaterial::alpha_cutoff", key, &[value])
    }

    #[lsp_doc("docs/api/scene/unlit_material/alpha_mode.md")]
    pub fn alpha_mode(&self, mode: AlphaMode) -> Self {
        self.shader.object.set_alpha_mode(mode);
        let key = "material.alpha_mode_flag";
        let flag = UniformInput::Number(mode.flag().into());
        if let Err(error) = write_values(&self.shader, vec![(key.to_string(), flag)]) {
            log::warn!("UnlitMaterial setter '{key}' did not apply: {error}");
        }
        self.clone()
    }

    #[lsp_doc("docs/api/scene/unlit_material/double_sided.md")]
    pub fn double_sided(&self, value: bool) -> Self {
        self.shader.object.set_double_sided(value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/unlit_material/base_color_map.md")]
    pub fn base_color_map(&self) -> MaterialMap {
        MaterialMap::new(self.shader.clone(), MapSlot::BaseColor, "UnlitMaterial")
    }
}

impl UnlitMaterial {
    /// Writes the factor under `key` from `numbers`, all of them finite, as `call` takes it.
    fn set_factor(&self, call: &str, key: &str, numbers: &[f32]) -> Result<Self, MaterialError> {
        let item = (numbers.len() == 1).then_some("value");
        check_finite(call, item, numbers)?;
        write_values(
            &self.shader,
            vec![(key.to_string(), UniformInput::from(numbers))],
        )?;
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests;
