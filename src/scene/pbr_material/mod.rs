//! `PbrMaterial`: glTF 2.0's metallic-roughness model on FragmentColor's built-in PBR shader.
//!
//! `PbrMaterial::new()` ships the default physically-based shader and the glTF 2.0
//! PBR-MR field set as builder-style setters. Its five maps (`base_color_map()`,
//! `metallic_roughness_map()`, `normal_map()`, `occlusion_map()` and `emissive_map()`)
//! start unbound; the renderer fills them with neutral 1×1 fallbacks at draw time
//! when the caller hasn't supplied a texture, so a material that sets no maps still
//! renders correctly. Each map samples its own UV set through its own
//! `KHR_texture_transform`, which its `MaterialMap` handle sets.
//!
//! The number setters return a `MaterialError` for a number that isn't finite, and a
//! refused call changes nothing: `write` checks every value of a call against the
//! shader before the first one lands.
//!
//! Per-Model transform is *not* a material uniform. It rides on a Pass-owned
//! per-instance vertex-attribute buffer that the renderer builds from the Model
//! entries `Pass::add` queues. That means many Models can share one material's
//! Shader without colliding on a `mesh.model` slot; the renderer batches them
//! by pipeline hash and pays one bind-group setup for the whole group.
//! `PbrMaterial` itself is `Clone` (shallow, Arc-share): cloning gives you another
//! handle to the same underlying shader state, not an independent copy.

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

/// Assembled vertex+fragment+uniform block for the built-in PBR shader. The
/// `mesh/transform` and `material/pbr` snippets are pulled in as composition
/// parts at construction time. They declare no bindings of their own, only
/// helper functions, so they slot in cleanly alongside this main body.
const PBR_MAIN: &str = include_str!("pbr_main.wgsl");

/// The `material/pbr` registry snippet, baked in directly rather than resolved through the
/// shader registry, as `MESH_TRANSFORM` and `LINEAR_TO_SRGB` are (see `scene::material`).
const MATERIAL_PBR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/material/pbr.wgsl"
));

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/pbr_material/pbr_material.md")]
pub struct PbrMaterial {
    pub(crate) shader: Shader,
}

crate::impl_fc_kind!(PbrMaterial, "PbrMaterial");

impl Default for PbrMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl PbrMaterial {
    /// Build a material with FragmentColor's default physically-based shader.
    ///
    /// Infallible on every supported build configuration. The snippets the shader
    /// composes from (`mesh/transform`, `material/pbr` and `color/linear_to_srgb`) are baked
    /// into the binary directly, so construction needs no shader registry and
    /// no network. It works the same on native, mobile, and the slim web
    /// build (`--no-default-features`), whose synchronous constructor cannot
    /// fetch over `fetch`.
    #[lsp_doc("docs/api/scene/pbr_material/new.md")]
    pub fn new() -> Self {
        // SAFETY-justified expect: the six parts are WGSL source compiled into the binary at build
        // time. The only way this fails is a malformed shader file in the repo, which is a build-time
        // programmer error caught by the doctests and healthchecks, not a runtime fault.
        let shader = Shader::new([
            MESH_TRANSFORM,
            MATERIAL_PBR,
            LINEAR_TO_SRGB,
            LIGHT,
            LIGHT_LOOPS,
            PBR_MAIN,
        ])
        .expect("SAFETY: built-in PBR shader parts are compiled-in WGSL source; a parse failure is a build-time error, not a runtime fault");
        let material = Self { shader };
        material.apply_defaults();
        material.shader.object.set_alpha_mode(AlphaMode::default());
        material
    }

    /// A material over `shader`, for tests that need a small shader in place of the PBR one.
    #[cfg(test)]
    pub(crate) fn wrap(shader: Shader) -> Self {
        Self { shader }
    }

    #[lsp_doc("docs/api/scene/pbr_material/shader.md")]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    fn apply_defaults(&self) {
        // glTF 2.0 PBR-MR factor defaults, with two ergonomic deviations: we
        // default `metallic=0` and `roughness=1` instead of glTF's
        // `metallic=1, roughness=1` (which renders as dark gunmetal: fine for
        // a texture-driven glTF material, but a bad first
        // frame for someone calling `PbrMaterial::new()` and rendering a
        // flat-color cube).
        let _ = self
            .shader
            .set("material.base_color", [1.0_f32, 1.0, 1.0, 1.0]);
        let _ = self.shader.set("material.metallic", 0.0_f32);
        let _ = self.shader.set("material.roughness", 1.0_f32);
        let _ = self.shader.set("material.normal_scale", 1.0_f32);
        let _ = self.shader.set("material.occlusion_strength", 1.0_f32);
        let _ = self.shader.set("material.emissive", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("material.alpha_cutoff", 0.5_f32);
        // 0 == Opaque (matches AlphaMode::default().flag()). The Mask branch
        // gates on this flag in fs_main; other modes ignore it.
        let _ = self.shader.set("material.alpha_mode_flag", 0_u32);
        // Every map samples UV0 through the identity transform.
        for slot in MapSlot::ALL {
            let _ = self.map(slot).set_uv(UvSet::First, UvTransform::default());
        }
        let identity = [
            [1.0_f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let _ = self.shader.set("camera.view_proj", identity);
        let _ = self.shader.set("camera.position", [0.0_f32, 0.0, 0.0]);
        // A white light from straight above lights a fresh material until the
        // first Light reaches it. Ambient is a small gray tint so unlit faces
        // don't read as pitch-black.
        self.shader.object.set_placeholder();
        let _ = self.shader.set("lights.ambient", [0.03_f32, 0.03, 0.03]);
    }

    // --- factor setters ---
    //
    // Every setter takes `&self` and returns `Self` (an Arc-cloned handle
    // to the same material), the same shape Light and Camera use, so it
    // chains and stays callable on a borrowed material. A number setter
    // returns it in a `Result`, refusing a value the material can't use.

    #[lsp_doc("docs/api/scene/pbr_material/base_color.md")]
    pub fn base_color(&self, color: [f32; 4]) -> Result<Self, MaterialError> {
        self.set_factor("PbrMaterial::base_color", "material.base_color", &color)
    }

    #[lsp_doc("docs/api/scene/pbr_material/metallic.md")]
    pub fn metallic(&self, value: f32) -> Result<Self, MaterialError> {
        self.set_factor("PbrMaterial::metallic", "material.metallic", &[value])
    }

    #[lsp_doc("docs/api/scene/pbr_material/roughness.md")]
    pub fn roughness(&self, value: f32) -> Result<Self, MaterialError> {
        self.set_factor("PbrMaterial::roughness", "material.roughness", &[value])
    }

    #[lsp_doc("docs/api/scene/pbr_material/normal_scale.md")]
    pub fn normal_scale(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.normal_scale";
        self.set_factor("PbrMaterial::normal_scale", key, &[value])
    }

    #[lsp_doc("docs/api/scene/pbr_material/occlusion_strength.md")]
    pub fn occlusion_strength(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.occlusion_strength";
        self.set_factor("PbrMaterial::occlusion_strength", key, &[value])
    }

    #[lsp_doc("docs/api/scene/pbr_material/emissive.md")]
    pub fn emissive(&self, factor: [f32; 3]) -> Result<Self, MaterialError> {
        self.set_factor("PbrMaterial::emissive", "material.emissive", &factor)
    }

    #[lsp_doc("docs/api/scene/pbr_material/alpha_cutoff.md")]
    pub fn alpha_cutoff(&self, value: f32) -> Result<Self, MaterialError> {
        let key = "material.alpha_cutoff";
        self.set_factor("PbrMaterial::alpha_cutoff", key, &[value])
    }

    #[lsp_doc("docs/api/scene/pbr_material/uv_transform.md")]
    pub fn uv_transform(
        &self,
        offset: [f32; 2],
        scale: [f32; 2],
        rotation: f32,
    ) -> Result<Self, MaterialError> {
        let transform = UvTransform::new(offset, scale, rotation);
        let transform = transform.check("PbrMaterial::uv_transform")?;
        let values = MapSlot::ALL
            .into_iter()
            .flat_map(|slot| slot.build_transform(transform))
            .collect();
        write_values(&self.shader, values)?;
        Ok(self.clone())
    }

    // --- pipeline-state flags ---

    #[lsp_doc("docs/api/scene/pbr_material/alpha_mode.md")]
    pub fn alpha_mode(&self, mode: AlphaMode) -> Self {
        self.shader.object.set_alpha_mode(mode);
        self.write_flag(mode);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/pbr_material/double_sided.md")]
    pub fn double_sided(&self, value: bool) -> Self {
        self.shader.object.set_double_sided(value);
        self.clone()
    }

    // --- maps ---
    //
    // Each call returns a handle to one map of this material, whose `texture`, `uv` and
    // `transform` set what the map samples. Color-space hints: `base_color` and `emissive`
    // are sRGB per glTF 2.0, and the three maps that hold data (`metallic_roughness`,
    // `normal`, `occlusion`) are linear.

    #[lsp_doc("docs/api/scene/pbr_material/base_color_map.md")]
    pub fn base_color_map(&self) -> MaterialMap {
        self.map(MapSlot::BaseColor)
    }

    #[lsp_doc("docs/api/scene/pbr_material/metallic_roughness_map.md")]
    pub fn metallic_roughness_map(&self) -> MaterialMap {
        self.map(MapSlot::MetallicRoughness)
    }

    #[lsp_doc("docs/api/scene/pbr_material/normal_map.md")]
    pub fn normal_map(&self) -> MaterialMap {
        self.map(MapSlot::Normal)
    }

    #[lsp_doc("docs/api/scene/pbr_material/occlusion_map.md")]
    pub fn occlusion_map(&self) -> MaterialMap {
        self.map(MapSlot::Occlusion)
    }

    #[lsp_doc("docs/api/scene/pbr_material/emissive_map.md")]
    pub fn emissive_map(&self) -> MaterialMap {
        self.map(MapSlot::Emissive)
    }
}

impl PbrMaterial {
    /// The handle to the map in `slot`.
    pub(crate) fn map(&self, slot: MapSlot) -> MaterialMap {
        MaterialMap::new(self.shader.clone(), slot, "PbrMaterial")
    }

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

    /// Writes the flag the PBR shader reads `mode` from.
    fn write_flag(&self, mode: AlphaMode) {
        let key = "material.alpha_mode_flag";
        let flag = UniformInput::Number(mode.flag().into());
        if let Err(error) = write_values(&self.shader, vec![(key.to_string(), flag)]) {
            log::warn!("PbrMaterial setter '{key}' did not apply: {error}");
        }
    }
}

#[cfg(test)]
mod tests;
