//! `ToonMaterial`: cel shading, with diffuse light cut into bands or read from a ramp,
//! a hard highlight and a rim.
//!
//! It reads the `camera` and `lights` roots that `PbrMaterial` reads, through the same light
//! loops, so the clustered lights light it as they light a PBR model. It takes the same vertex
//! layout as `PbrMaterial`: a `Vertex::pbr` mesh, or a mesh the glTF loader built. Its two maps,
//! `base_color_map()` and `ramp_map()`, start unbound, and the renderer fills them with a white
//! 1×1 fallback until the caller sets one. The shader reads a ramp one texel wide as no ramp, so
//! the material draws its `bands` until a ramp is set.
//!
//! `ToonMaterial` is `Clone` (shallow, Arc-share): cloning gives another handle to the same
//! shader state, as with `PbrMaterial`.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::core::UniformInput;
use crate::scene::material::{LIGHT, LIGHT_LOOPS, LINEAR_TO_SRGB, MESH_TRANSFORM, write_values};
use crate::scene::{MapSlot, UvSet, UvTransform, check_finite};
use crate::{AlphaMode, MaterialError, MaterialMap, Shader};

mod platform;

/// The main body of the toon shader, which composes the snippets and light loops that
/// `PbrMaterial` composes too.
const TOON_MAIN: &str = include_str!("toon_main.wgsl");

/// The bands a new material steps its diffuse light in.
const DEFAULT_BANDS: u32 = 3;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/toon_material/toon_material.md")]
pub struct ToonMaterial {
    pub(crate) shader: Shader,
}

crate::impl_fc_kind!(ToonMaterial, "ToonMaterial");

impl Default for ToonMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl ToonMaterial {
    /// Build a white, opaque, single-sided toon material with three bands and no
    /// highlight or rim.
    ///
    /// Infallible on every supported build configuration, as `PbrMaterial::new` is: the
    /// snippets its shader composes from are baked into the binary.
    #[lsp_doc("docs/api/scene/toon_material/new.md")]
    pub fn new() -> Self {
        // SAFETY-justified expect: the parts are WGSL source compiled into the binary at build
        // time. The only way this fails is a malformed shader file in the repo, which is a
        // build-time programmer error caught by the tests, not a runtime fault.
        let shader = Shader::new([MESH_TRANSFORM, LINEAR_TO_SRGB, LIGHT, LIGHT_LOOPS, TOON_MAIN])
            .expect("SAFETY: built-in toon shader parts are compiled-in WGSL source; a parse failure is a build-time error, not a runtime fault");
        let material = Self { shader };
        material.apply_defaults();
        material.shader.object.set_alpha_mode(AlphaMode::default());
        material
    }

    #[lsp_doc("docs/api/scene/toon_material/shader.md")]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    fn apply_defaults(&self) {
        let _ = self
            .shader
            .set("material.base_color", [1.0_f32, 1.0, 1.0, 1.0]);
        // Black turns the highlight and the rim off, and their sizes wait for a color.
        let _ = self.shader.set("material.highlight", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("material.highlight_size", 0.05_f32);
        let _ = self.shader.set("material.rim", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("material.rim_width", 0.2_f32);
        let _ = self.shader.set("material.bands", DEFAULT_BANDS);
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
        // A white light from straight above lights a fresh material until the
        // first Light reaches it, as on PbrMaterial.
        self.shader.object.set_placeholder();
        let _ = self.shader.set("lights.ambient", [0.03_f32, 0.03, 0.03]);
    }

    #[lsp_doc("docs/api/scene/toon_material/base_color.md")]
    pub fn base_color(&self, color: [f32; 4]) -> Result<Self, MaterialError> {
        self.set_factor("ToonMaterial::base_color", "material.base_color", &color)
    }

    #[lsp_doc("docs/api/scene/toon_material/bands.md")]
    pub fn bands(&self, count: u32) -> Result<Self, MaterialError> {
        if count == 0 {
            let call = "ToonMaterial::bands".to_string();
            return Err(MaterialError::Bands { call, count });
        }
        let value = UniformInput::Number(count.into());
        write_values(&self.shader, vec![("material.bands".to_string(), value)])?;
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/toon_material/highlight.md")]
    pub fn highlight(&self, color: [f32; 3]) -> Result<Self, MaterialError> {
        self.set_factor("ToonMaterial::highlight", "material.highlight", &color)
    }

    #[lsp_doc("docs/api/scene/toon_material/highlight_size.md")]
    pub fn highlight_size(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.highlight_size";
        self.set_factor("ToonMaterial::highlight_size", key, &[value])
    }

    #[lsp_doc("docs/api/scene/toon_material/rim.md")]
    pub fn rim(&self, color: [f32; 3]) -> Result<Self, MaterialError> {
        self.set_factor("ToonMaterial::rim", "material.rim", &color)
    }

    #[lsp_doc("docs/api/scene/toon_material/rim_width.md")]
    pub fn rim_width(&self, value: f32) -> Result<Self, MaterialError> {
        self.set_factor("ToonMaterial::rim_width", "material.rim_width", &[value])
    }

    #[lsp_doc("docs/api/scene/toon_material/alpha_cutoff.md")]
    pub fn alpha_cutoff(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.alpha_cutoff";
        self.set_factor("ToonMaterial::alpha_cutoff", key, &[value])
    }

    #[lsp_doc("docs/api/scene/toon_material/alpha_mode.md")]
    pub fn alpha_mode(&self, mode: AlphaMode) -> Self {
        self.shader.object.set_alpha_mode(mode);
        let key = "material.alpha_mode_flag";
        let flag = UniformInput::Number(mode.flag().into());
        if let Err(error) = write_values(&self.shader, vec![(key.to_string(), flag)]) {
            log::warn!("ToonMaterial setter '{key}' did not apply: {error}");
        }
        self.clone()
    }

    #[lsp_doc("docs/api/scene/toon_material/double_sided.md")]
    pub fn double_sided(&self, value: bool) -> Self {
        self.shader.object.set_double_sided(value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/toon_material/base_color_map.md")]
    pub fn base_color_map(&self) -> MaterialMap {
        MaterialMap::new(self.shader.clone(), MapSlot::BaseColor, "ToonMaterial")
    }

    #[lsp_doc("docs/api/scene/toon_material/ramp_map.md")]
    pub fn ramp_map(&self) -> MaterialMap {
        MaterialMap::new(self.shader.clone(), MapSlot::Ramp, "ToonMaterial")
    }
}

impl ToonMaterial {
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
