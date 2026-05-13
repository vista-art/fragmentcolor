//! Material — PBR data + `Shader` bundle.
//!
//! `Material::pbr()` ships FragmentColor's default physically-based shader
//! and the glTF 2.0 PBR-MR field set as builder-style setters.
//! `Material::custom(shader)` wraps an arbitrary `Shader` and reuses the same
//! setter API for any uniform the custom shader happens to declare under
//! matching paths.
//!
//! Per-Model transform is *not* a Material uniform — it rides on the per-
//! instance vertex attribute stream written by `Model::sync_transform`. That
//! means many Models can share one Material's Shader without colliding on a
//! `mesh.model` uniform; the renderer batches them by pipeline hash and pays
//! one bind-group setup for the whole group. Material itself is `Clone`
//! (shallow, Arc-share) — cloning gives you another handle to the same
//! underlying shader state, not an independent copy.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::{Shader, ShaderError, UniformData};

mod platform;

/// Assembled vertex+fragment+uniform block for the built-in PBR shader. The
/// `mesh/transform` and `material/pbr` registry snippets are pulled in as
/// composition parts at construction time — they declare no bindings of
/// their own, only helper functions, so they slot in cleanly alongside this
/// main body.
const PBR_MAIN: &str = include_str!("pbr_main.wgsl");

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/material/material.md")]
pub struct Material {
    pub(crate) shader: Shader,
}

crate::impl_fc_kind!(Material, "Material");

impl Material {
    /// Build a Material with FragmentColor's default physically-based shader.
    ///
    /// Returns `Err(ShaderError)` only when the registry slugs `mesh/transform`
    /// and `material/pbr` can't be resolved — i.e. on a build that opted out
    /// of both the `shaders-mesh` and `shaders-material` Cargo features
    /// (the default `shaders-all` includes both). For web slim builds
    /// (`--no-default-features`), enable them explicitly:
    ///
    /// ```text
    /// --features=shaders-mesh,shaders-material
    /// ```
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr() -> Result<Self, ShaderError> {
        let shader = Shader::new(["mesh/transform", "material/pbr", PBR_MAIN])?;
        let material = Self { shader };
        material.apply_defaults();
        Ok(material)
    }

    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom(shader: Shader) -> Self {
        Self { shader }
    }

    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    fn apply_defaults(&self) {
        // glTF 2.0 PBR-MR factor defaults, with two ergonomic deviations: we
        // default `metallic=0` and `roughness=1` instead of glTF's
        // `metallic=1, roughness=1` (which renders as dark gunmetal under the
        // factors-only shader — fine if you're loading a texture-driven glTF
        // material, but a bad first-frame for someone calling `Material::pbr()`
        // and rendering a flat-color cube).
        let _ = self.shader.set("material.base_color", [1.0_f32, 1.0, 1.0, 1.0]);
        let _ = self.shader.set("material.metallic", 0.0_f32);
        let _ = self.shader.set("material.roughness", 1.0_f32);
        let _ = self.shader.set("material.normal_scale", 1.0_f32);
        let _ = self.shader.set("material.occlusion_strength", 1.0_f32);
        let _ = self.shader.set("material.emissive", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("material.alpha_cutoff", 0.5_f32);
        let identity = [
            [1.0_f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let _ = self.shader.set("camera.view_proj", identity);
        let _ = self.shader.set("camera.position", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("light.direction", [0.0_f32, -1.0, 0.0]);
        let _ = self.shader.set("light.color", [1.0_f32, 1.0, 1.0]);
    }

    // --- factor setters ---

    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color(self, color: [f32; 4]) -> Self {
        set_or_warn(&self.shader, "material.base_color", color);
        self
    }

    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic(self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.metallic", value);
        self
    }

    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness(self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.roughness", value);
        self
    }

    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale(self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.normal_scale", value);
        self
    }

    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength(self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.occlusion_strength", value);
        self
    }

    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive(self, factor: [f32; 3]) -> Self {
        set_or_warn(&self.shader, "material.emissive", factor);
        self
    }

    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff(self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.alpha_cutoff", value);
        self
    }

    // --- texture setters ---
    //
    // Texture setters store the texture under the canonical glTF binding name.
    // The factors-only default PBR shader doesn't declare these bindings yet,
    // so the calls log-warn and return self. They become effective with
    // `Material::custom(shader_that_samples_textures)` today, and with
    // `Material::pbr` in the follow-up that adds texture sampling to the
    // default shader (see CHANGELOG).

    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture(self, texture: &crate::texture::Texture) -> Self {
        set_texture_or_warn(&self.shader, "base_color_map", texture);
        self
    }

    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture(self, texture: &crate::texture::Texture) -> Self {
        set_texture_or_warn(&self.shader, "metallic_roughness_map", texture);
        self
    }

    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture(self, texture: &crate::texture::Texture) -> Self {
        set_texture_or_warn(&self.shader, "normal_map", texture);
        self
    }

    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture(self, texture: &crate::texture::Texture) -> Self {
        set_texture_or_warn(&self.shader, "occlusion_map", texture);
        self
    }

    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture(self, texture: &crate::texture::Texture) -> Self {
        set_texture_or_warn(&self.shader, "emissive_map", texture);
        self
    }
}

pub(crate) fn set_or_warn<V: Into<UniformData>>(shader: &Shader, key: &str, value: V) {
    if let Err(e) = shader.set(key, value) {
        log::warn!("Material setter '{key}' did not apply: {e}");
    }
}

pub(crate) fn set_texture_or_warn(shader: &Shader, key: &str, texture: &crate::texture::Texture) {
    let meta = crate::texture::TextureMeta::with_id_only(*texture.id());
    if let Err(e) = shader.set(key, UniformData::Texture(meta)) {
        log::warn!("Material texture '{key}' did not apply: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pbr_seeds_default_uniforms() {
        let mat = Material::pbr().expect("pbr template compiles");
        let base: [f32; 4] = mat.shader().get("material.base_color").expect("base_color");
        assert_eq!(base, [1.0, 1.0, 1.0, 1.0]);
        let metallic: f32 = mat.shader().get("material.metallic").expect("metallic");
        assert_eq!(metallic, 0.0);
        let roughness: f32 = mat.shader().get("material.roughness").expect("roughness");
        assert_eq!(roughness, 1.0);
    }

    #[test]
    fn builder_setters_update_uniforms() {
        let mat = Material::pbr()
            .expect("pbr")
            .base_color([0.4, 0.7, 0.2, 0.8])
            .metallic(0.6)
            .roughness(0.25)
            .emissive([0.0, 0.1, 0.0]);

        let base: [f32; 4] = mat.shader().get("material.base_color").unwrap();
        assert_eq!(base, [0.4, 0.7, 0.2, 0.8]);
        let metallic: f32 = mat.shader().get("material.metallic").unwrap();
        assert!((metallic - 0.6).abs() < 1.0e-6);
        let roughness: f32 = mat.shader().get("material.roughness").unwrap();
        assert!((roughness - 0.25).abs() < 1.0e-6);
        let emissive: [f32; 3] = mat.shader().get("material.emissive").unwrap();
        assert_eq!(emissive, [0.0, 0.1, 0.0]);
    }

    #[test]
    fn clone_shares_shader_state() {
        let original = Material::pbr().expect("pbr").base_color([1.0, 0.0, 0.0, 1.0]);
        let handle_b = original.clone();
        // Mutating one handle is visible on the other — the share is shallow
        // by design so renderer batching can see one pipeline for the pair.
        let _ = handle_b.shader().set("material.base_color", [0.0_f32, 1.0, 0.0, 1.0]);

        let original_color: [f32; 4] = original.shader().get("material.base_color").unwrap();
        let b_color: [f32; 4] = handle_b.shader().get("material.base_color").unwrap();
        assert_eq!(original_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(b_color, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn custom_wraps_arbitrary_shader_and_setters_no_op_silently() {
        // A shader with no `material.base_color` uniform.
        let shader = Shader::new(
            r#"
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
                let p = array<vec2<f32>, 3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0));
                return vec4<f32>(p[i], 0.0, 1.0);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0);
            }
            "#,
        )
        .expect("compile");

        let mat = Material::custom(shader).base_color([1.0, 0.5, 0.0, 1.0]);
        // Setter is best-effort; the shader doesn't declare material.base_color so
        // the value is silently dropped (with a debug log). What we care about is
        // that the call doesn't panic and the Material is still usable.
        let _ = mat;
    }
}
