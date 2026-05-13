//! Material — PBR data + `Shader` bundle.
//!
//! `Material::pbr(&renderer)` ships FragmentColor's default physically-based
//! shader and the glTF 2.0 PBR-MR field set as builder-style setters. It
//! takes the `Renderer` so the five glTF texture slots come pre-bound to
//! 1×1 defaults pulled from the renderer's lazy texture cache, leaving the
//! shader's bind groups complete the moment the Material is constructed.
//! `Material::custom(shader)` wraps an arbitrary `Shader` and reuses the
//! same setter API for any uniform the custom shader happens to declare
//! under matching paths.
//!
//! Per-Model transform is *not* a Material uniform — it rides on a Pass-
//! owned per-instance vertex-attribute buffer that the renderer builds from
//! `Pass::add_model` entries. That means many Models can share one Material's
//! Shader without colliding on a `mesh.model` slot; the renderer batches them
//! by pipeline hash and pays one bind-group setup for the whole group.
//! Material itself is `Clone` (shallow, Arc-share) — cloning gives you another
//! handle to the same underlying shader state, not an independent copy.

use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::{Renderer, RendererError, Shader, UniformData};

mod alpha_mode;
pub use alpha_mode::AlphaMode;

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
    /// Pipeline-state alpha mode. Mutable through `&self` so the cross-
    /// language bindings (which can't take `self` by value) and the Rust
    /// builder both write through to the same slot — and so cloned handles
    /// see each other's updates, matching the shallow-Clone share semantics
    /// the rest of Material already follows.
    pub(crate) alpha_mode: Arc<RwLock<AlphaMode>>,
    /// Pipeline-state double-sided flag. Same share semantics as
    /// `alpha_mode`.
    pub(crate) double_sided: Arc<RwLock<bool>>,
}

crate::impl_fc_kind!(Material, "Material");

impl Material {
    /// Build a Material with FragmentColor's default physically-based shader.
    ///
    /// Returns `Err(RendererError)` on two paths:
    /// 1. `ShaderError` (wrapped via `RendererError::ShaderError`) when the
    ///    registry slugs `mesh/transform` and `material/pbr` can't be
    ///    resolved — i.e. on a build that opted out of both the
    ///    `shaders-mesh` and `shaders-material` Cargo features (the default
    ///    `shaders-all` includes both). For web slim builds
    ///    (`--no-default-features`), enable them explicitly:
    ///
    ///    ```text
    ///    --features=shaders-mesh,shaders-material
    ///    ```
    /// 2. Any error returned by the renderer when lazy-creating the shared
    ///    default-PBR textures on the first call (adapter init, device
    ///    init, texture upload). Subsequent `Material::pbr` calls hit the
    ///    cached bundle and only carry the shader-error path.
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub async fn pbr(renderer: &Renderer) -> Result<Self, RendererError> {
        let shader = Shader::new(["mesh/transform", "material/pbr", PBR_MAIN])?;
        let material = Self {
            shader,
            alpha_mode: Arc::new(RwLock::new(AlphaMode::default())),
            double_sided: Arc::new(RwLock::new(false)),
        };
        material.apply_defaults();
        // Seed the ShaderObject back-references. The default ShaderObject
        // double_sided is `true` (preserves the pre-Material no-cull behaviour
        // for raw Shader callers); Material::pbr flips it to `false` to follow
        // glTF 2.0's single-sided default.
        material.shader.object.set_alpha_mode(AlphaMode::default());
        material.shader.object.set_double_sided(false);
        let defaults = renderer.default_pbr_textures().await?;
        material.apply_default_textures(&defaults);
        Ok(material)
    }

    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom(shader: Shader) -> Self {
        let mat = Self {
            shader,
            alpha_mode: Arc::new(RwLock::new(AlphaMode::default())),
            double_sided: Arc::new(RwLock::new(false)),
        };
        // Same single-sided default as Material::pbr — opting into Material
        // semantics means opting into glTF 2.0 defaults, even when the
        // underlying Shader was originally constructed standalone.
        mat.shader.object.set_alpha_mode(AlphaMode::default());
        mat.shader.object.set_double_sided(false);
        mat
    }

    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    fn apply_defaults(&self) {
        // glTF 2.0 PBR-MR factor defaults, with two ergonomic deviations: we
        // default `metallic=0` and `roughness=1` instead of glTF's
        // `metallic=1, roughness=1` (which renders as dark gunmetal — fine if
        // you're loading a texture-driven glTF material, but a bad first-
        // frame for someone calling `Material::pbr(&renderer).await?` and
        // rendering a flat-color cube).
        let _ = self.shader.set("material.base_color", [1.0_f32, 1.0, 1.0, 1.0]);
        let _ = self.shader.set("material.metallic", 0.0_f32);
        let _ = self.shader.set("material.roughness", 1.0_f32);
        let _ = self.shader.set("material.normal_scale", 1.0_f32);
        let _ = self.shader.set("material.occlusion_strength", 1.0_f32);
        let _ = self.shader.set("material.emissive", [0.0_f32, 0.0, 0.0]);
        let _ = self.shader.set("material.alpha_cutoff", 0.5_f32);
        // 0 == Opaque (matches AlphaMode::default().flag()). The Mask branch
        // gates on this flag in fs_main; other modes ignore it.
        let _ = self.shader.set("material.alpha_mode_flag", 0_u32);
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

    /// Bind the renderer's shared 1×1 fallback textures into the canonical
    /// glTF map slots so the default PBR shader's bind group is complete
    /// the moment the Material is constructed. Any user-supplied texture
    /// passed to `base_color_texture` / `metallic_roughness_texture` / etc.
    /// later overrides the matching slot.
    fn apply_default_textures(&self, defaults: &crate::renderer::DefaultPbrTextures) {
        set_texture_or_warn(&self.shader, "base_color_map", &defaults.base_color);
        set_texture_or_warn(
            &self.shader,
            "metallic_roughness_map",
            &defaults.metallic_roughness,
        );
        set_texture_or_warn(&self.shader, "normal_map", &defaults.normal);
        set_texture_or_warn(&self.shader, "occlusion_map", &defaults.occlusion);
        set_texture_or_warn(&self.shader, "emissive_map", &defaults.emissive);
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

    // --- pipeline-state flags ---

    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode(self, mode: AlphaMode) -> Self {
        *self.alpha_mode.write() = mode;
        self.shader.object.set_alpha_mode(mode);
        set_or_warn(&self.shader, "material.alpha_mode_flag", mode.flag());
        self
    }

    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided(self, value: bool) -> Self {
        *self.double_sided.write() = value;
        self.shader.object.set_double_sided(value);
        self
    }

    // --- texture setters ---
    //
    // Each setter overrides the corresponding 1×1 default the renderer seeded
    // at construction time. The default PBR shader samples every slot in
    // `fs_main` and combines it with the matching factor per the glTF 2.0
    // spec; with a `Material::custom(shader)` they're still best-effort under
    // the same binding names, no-op-ing for shaders that don't declare them.

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
    use crate::mesh::{Mesh, Vertex};
    use crate::{Pass, Target};

    fn pbr_material(renderer: &Renderer) -> Material {
        pollster::block_on(Material::pbr(renderer)).expect("pbr template compiles")
    }

    #[test]
    fn pbr_seeds_default_uniforms() {
        let renderer = Renderer::new();
        let mat = pbr_material(&renderer);
        let base: [f32; 4] = mat.shader().get("material.base_color").expect("base_color");
        assert_eq!(base, [1.0, 1.0, 1.0, 1.0]);
        let metallic: f32 = mat.shader().get("material.metallic").expect("metallic");
        assert_eq!(metallic, 0.0);
        let roughness: f32 = mat.shader().get("material.roughness").expect("roughness");
        assert_eq!(roughness, 1.0);
    }

    #[test]
    fn builder_setters_update_uniforms() {
        let renderer = Renderer::new();
        let mat = pbr_material(&renderer)
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
        let renderer = Renderer::new();
        let original = pbr_material(&renderer).base_color([1.0, 0.0, 0.0, 1.0]);
        let handle_b = original.clone();
        let _ = handle_b
            .shader()
            .set("material.base_color", [0.0_f32, 1.0, 0.0, 1.0]);

        let original_color: [f32; 4] = original.shader().get("material.base_color").unwrap();
        let b_color: [f32; 4] = handle_b.shader().get("material.base_color").unwrap();
        assert_eq!(original_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(b_color, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn pbr_seeds_default_texture_bindings() {
        let renderer = Renderer::new();
        let mat = pbr_material(&renderer);
        for slot in [
            "base_color_map",
            "metallic_roughness_map",
            "normal_map",
            "occlusion_map",
            "emissive_map",
        ] {
            let data = mat
                .shader()
                .object
                .get_uniform_data(slot)
                .unwrap_or_else(|_| panic!("default-bound slot '{slot}' missing on Material::pbr"));
            assert!(
                matches!(data, UniformData::Texture(_)),
                "slot '{slot}' should hold a Texture uniform, got {:?}",
                data
            );
        }
    }

    #[test]
    fn alpha_mode_setter_threads_to_shader_back_reference() {
        let renderer = Renderer::new();
        let mat = pbr_material(&renderer).alpha_mode(AlphaMode::Mask);
        assert_eq!(*mat.alpha_mode.read(), AlphaMode::Mask);
        // ShaderObject back-reference picks up the new mode for the pipeline
        // cache key; the alpha_mode_flag uniform picks it up too so fs_main's
        // Mask discard branch fires.
        let flag: u32 = mat.shader().get("material.alpha_mode_flag").unwrap();
        assert_eq!(flag, AlphaMode::Mask.flag());
    }

    #[test]
    fn double_sided_setter_threads_to_shader_back_reference() {
        let renderer = Renderer::new();
        let mat = pbr_material(&renderer).double_sided(true);
        assert!(*mat.double_sided.read());
        assert!(*mat.shader.object.double_sided.read());
    }

    #[test]
    fn custom_wraps_arbitrary_shader_and_setters_no_op_silently() {
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
        let _ = mat;
    }

    fn pbr_triangle_mesh() -> Mesh {
        let mesh = Mesh::new();
        for (p, uv) in [
            ([0.0, 0.5, 0.0], [0.5, 1.0]),
            ([-0.5, -0.5, 0.0], [0.0, 0.0]),
            ([0.5, -0.5, 0.0], [1.0, 0.0]),
        ] {
            mesh.add_vertex(
                Vertex::new(p)
                    .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                    .set(Vertex::UV0, uv),
            );
        }
        mesh
    }

    #[test]
    fn pbr_with_no_user_textures_renders_with_defaults() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");
            let mat = Material::pbr(&renderer)
                .await
                .expect("pbr")
                .base_color([0.6, 0.2, 0.8, 1.0]);
            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat);
            let pass = Pass::new("defaults-only");
            pass.add_model(&model).expect("add_model");
            renderer
                .render(&pass, &target)
                .expect("render with all default textures");
            let img = target.get_image().await;
            assert_eq!(img.len(), 8 * 8 * 4);
        });
    }

    #[test]
    fn pbr_samples_base_color_texture() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");
            #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255,   0,   0, 255,    0, 255,   0, 255,
                  0,   0, 255, 255,  255, 255, 255, 255,
            ];
            let tex = renderer
                .create_texture((&pixels[..], crate::Size::from((2u32, 2u32))))
                .await
                .expect("base_color texture");
            tex.set_sampler_options(crate::texture::SamplerOptions {
                smooth: false,
                ..Default::default()
            });
            let mat = Material::pbr(&renderer)
                .await
                .expect("pbr")
                .base_color([1.0, 1.0, 1.0, 1.0])
                .base_color_texture(&tex);

            mat.shader().set("camera.position", [0.0_f32, 0.0, 1.0]).ok();
            mat.shader().set("light.direction", [0.0_f32, 0.0, -1.0]).ok();
            mat.shader().set("light.color", [1.0_f32, 1.0, 1.0]).ok();

            let mesh = Mesh::new();
            for (pos, uv) in [
                ([-1.0_f32, 1.0, 0.0], [0.0_f32, 1.0]),
                ([-1.0, -1.0, 0.0], [0.0, 0.0]),
                ([1.0, -1.0, 0.0], [1.0, 0.0]),
            ] {
                mesh.add_vertex(
                    Vertex::new(pos)
                        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                        .set(Vertex::UV0, uv),
                );
            }
            let model = crate::scene::Model::new(mesh, mat);
            let pass = Pass::new("textured-triangle");
            pass.add_model(&model).expect("add_model");
            renderer
                .render(&pass, &target)
                .expect("render textured triangle");
            let img = target.get_image().await;
            assert_eq!(img.len(), 16 * 16 * 4);

            let mut has_red = false;
            let mut has_blue = false;
            for px in img.chunks_exact(4) {
                if px[0] > 80 && px[1] < 60 && px[2] < 60 {
                    has_red = true;
                }
                if px[2] > 80 && px[0] < 60 && px[1] < 60 {
                    has_blue = true;
                }
            }
            assert!(
                has_red && has_blue,
                "expected sampled base_color texture to contribute red and blue pixels; got neither (has_red={has_red}, has_blue={has_blue})"
            );
        });
    }
}
