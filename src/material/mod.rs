//! Material — PBR data + `Shader` bundle.
//!
//! `Material::pbr()` ships FragmentColor's default physically-based shader
//! and the glTF 2.0 PBR-MR field set as builder-style setters. The five
//! glTF texture slots (`base_color_map`, `metallic_roughness_map`,
//! `normal_map`, `occlusion_map`, `emissive_map`) start unbound; the
//! renderer fills them with neutral 1×1 fallbacks at draw time when the
//! caller hasn't supplied a texture, so a Material that sets no maps still
//! renders correctly. `Material::custom(shader)` wraps an arbitrary
//! `Shader` and reuses the same setter API for any uniform the custom
//! shader happens to declare under matching paths.
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

use crate::{Shader, ShaderError, UniformData};

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
    /// Returns `Err(ShaderError)` only when the registry slugs
    /// `mesh/transform` and `material/pbr` can't be resolved — i.e. on a
    /// build that opted out of both the `shaders-mesh` and
    /// `shaders-material` Cargo features (the default `shaders-all`
    /// includes both). For web slim builds (`--no-default-features`),
    /// enable them explicitly:
    ///
    /// ```text
    /// --features=shaders-mesh,shaders-material
    /// ```
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr() -> Result<Self, ShaderError> {
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
        // frame for someone calling `Material::pbr()?` and rendering a
        // flat-color cube).
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
        // KHR_texture_transform defaults — identity (scale = 1, offset = 0,
        // rotation = 0), so every UV passes through unchanged. The
        // `material.uv_transform(...)` builder overrides these.
        let _ = self.shader.set("material.uv_offset", [0.0_f32, 0.0]);
        let _ = self.shader.set("material.uv_scale", [1.0_f32, 1.0]);
        let _ = self.shader.set("material.uv_rotation", 0.0_f32);
        let identity = [
            [1.0_f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let _ = self.shader.set("camera.view_proj", identity);
        let _ = self.shader.set("camera.position", [0.0_f32, 0.0, 0.0]);
        // Default lighting: one directional light at slot 0 (sun-style
        // angled top-down white). `lights.count = 1` so a freshly-built
        // Material renders correctly without any Light attached; the
        // first `pass.add(&light)` writes to slot 0 too, overwriting the
        // defaults. Subsequent Lights take slots 1, 2, … up to
        // `PBR_MAX_LIGHTS` (8). Ambient is a small grey tint so unlit
        // faces don't read as pitch-black — matches the prior shader's
        // hardcoded `* 0.03` term.
        let _ = self.shader.set("lights.count", 1_u32);
        let _ = self
            .shader
            .set("lights.ambient", [0.03_f32, 0.03, 0.03]);
        let _ = self.shader.set("lights.lights[0].kind", 0_u32);
        let _ = self
            .shader
            .set("lights.lights[0].direction", [0.0_f32, -1.0, 0.0]);
        let _ = self
            .shader
            .set("lights.lights[0].position", [0.0_f32, 0.0, 0.0]);
        let _ = self
            .shader
            .set("lights.lights[0].color", [1.0_f32, 1.0, 1.0]);
        let _ = self
            .shader
            .set("lights.lights[0].intensity", 1.0_f32);
        let _ = self.shader.set("lights.lights[0].range", 0.0_f32);
        let _ = self
            .shader
            .set("lights.lights[0].inner_cone_cos", 1.0_f32);
        let _ = self.shader.set(
            "lights.lights[0].outer_cone_cos",
            std::f32::consts::FRAC_PI_4.cos(),
        );
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

    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform(self, offset: [f32; 2], scale: [f32; 2], rotation: f32) -> Self {
        set_or_warn(&self.shader, "material.uv_offset", offset);
        set_or_warn(&self.shader, "material.uv_scale", scale);
        set_or_warn(&self.shader, "material.uv_rotation", rotation);
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
    // Each setter accepts either an already-uploaded `Texture` (the eager
    // path — the underlying `From<&Texture> for TextureInput` produces a
    // `TextureData::CloneOf` that's set on the shader immediately) or any
    // `Into<TextureInput>` value (path, bytes, URL, …) for the lazy path.
    // Lazy inputs are queued on the Shader's pending-texture list and the
    // renderer drains them on first render or via the explicit
    // [`Renderer::load`](crate::Renderer::load) surface.
    //
    // Color-space hints: `base_color` and `emissive` are sRGB per glTF 2.0;
    // the three data-encoding maps (`metallic_roughness`, `normal`,
    // `occlusion`) are linear. The setters inject the right
    // `TextureFormat` only when the caller hasn't already picked one — pass
    // `(source, TextureFormat::...)` to override.

    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture(self, source: impl Into<crate::TextureInput>) -> Self {
        queue_texture_or_warn(
            &self.shader,
            "base_color_map",
            source.into(),
            Some(crate::TextureFormat::Rgba8UnormSrgb),
        );
        self
    }

    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture(self, source: impl Into<crate::TextureInput>) -> Self {
        queue_texture_or_warn(
            &self.shader,
            "metallic_roughness_map",
            source.into(),
            Some(crate::TextureFormat::Rgba8Unorm),
        );
        self
    }

    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture(self, source: impl Into<crate::TextureInput>) -> Self {
        queue_texture_or_warn(
            &self.shader,
            "normal_map",
            source.into(),
            Some(crate::TextureFormat::Rgba8Unorm),
        );
        self
    }

    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture(self, source: impl Into<crate::TextureInput>) -> Self {
        queue_texture_or_warn(
            &self.shader,
            "occlusion_map",
            source.into(),
            Some(crate::TextureFormat::Rgba8Unorm),
        );
        self
    }

    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture(self, source: impl Into<crate::TextureInput>) -> Self {
        queue_texture_or_warn(
            &self.shader,
            "emissive_map",
            source.into(),
            Some(crate::TextureFormat::Rgba8UnormSrgb),
        );
        self
    }
}

pub(crate) fn set_or_warn<V: Into<UniformData>>(shader: &Shader, key: &str, value: V) {
    if let Err(e) = shader.set(key, value) {
        log::warn!("Material setter '{key}' did not apply: {e}");
    }
}

/// Either set a texture uniform immediately (when the input wraps an
/// already-uploaded `Texture`) or queue the upload on the Shader for the
/// renderer to drain at load/render time. `srgb_hint` lets the calling
/// setter inject the glTF-correct color space when the caller didn't pick a
/// non-default format — pass `None` for shaders that don't care.
pub(crate) fn queue_texture_or_warn(
    shader: &Shader,
    key: &str,
    mut input: crate::TextureInput,
    srgb_hint: Option<crate::TextureFormat>,
) {
    if let Some(hint) = srgb_hint {
        if input.options.format == crate::TextureFormat::default() {
            input.options.format = hint;
        }
    }
    if let Err(e) = shader.object.queue_or_set_texture(key, input) {
        log::warn!("Material texture '{key}' did not apply: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{Mesh, Vertex};
    use crate::{Pass, Renderer, Target};

    #[test]
    fn pbr_seeds_default_uniforms() {
        let mat = Material::pbr().expect("pbr");
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
        let original = Material::pbr()
            .expect("pbr")
            .base_color([1.0, 0.0, 0.0, 1.0]);
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
    fn alpha_mode_setter_threads_to_shader_back_reference() {
        let mat = Material::pbr().expect("pbr").alpha_mode(AlphaMode::Mask);
        assert_eq!(*mat.alpha_mode.read(), AlphaMode::Mask);
        // ShaderObject back-reference picks up the new mode for the pipeline
        // cache key; the alpha_mode_flag uniform picks it up too so fs_main's
        // Mask discard branch fires.
        let flag: u32 = mat.shader().get("material.alpha_mode_flag").unwrap();
        assert_eq!(flag, AlphaMode::Mask.flag());
    }

    #[test]
    fn double_sided_setter_threads_to_shader_back_reference() {
        let mat = Material::pbr().expect("pbr").double_sided(true);
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
                    .set(Vertex::UV0, uv).set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0]).set(Vertex::UV1, [0.0, 0.0]).set(Vertex::TANGENT, [1.0, 0.0, 0.0, 1.0]),
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
            let mat = Material::pbr()
                .expect("pbr")
                .base_color([0.6, 0.2, 0.8, 1.0]);
            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat);
            let pass = Pass::new("defaults-only");
            pass.add(&model).expect("add_model");
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
            let mat = Material::pbr()
                .expect("pbr")
                .base_color([1.0, 1.0, 1.0, 1.0])
                .base_color_texture(&tex);

            mat.shader().set("camera.position", [0.0_f32, 0.0, 1.0]).ok();
            mat.shader().set("lights.lights[0].direction", [0.0_f32, 0.0, -1.0]).ok();
            mat.shader().set("lights.lights[0].color", [1.0_f32, 1.0, 1.0]).ok();

            let mesh = Mesh::new();
            for (pos, uv) in [
                ([-1.0_f32, 1.0, 0.0], [0.0_f32, 1.0]),
                ([-1.0, -1.0, 0.0], [0.0, 0.0]),
                ([1.0, -1.0, 0.0], [1.0, 0.0]),
            ] {
                mesh.add_vertex(
                    Vertex::new(pos)
                        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                        .set(Vertex::UV0, uv).set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0]).set(Vertex::UV1, [0.0, 0.0]).set(Vertex::TANGENT, [1.0, 0.0, 0.0, 1.0]),
                );
            }
            let model = crate::scene::Model::new(mesh, mat);
            let pass = Pass::new("textured-triangle");
            pass.add(&model).expect("add_model");
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

    #[test]
    fn base_color_texture_with_raw_bytes_is_pending_until_render() {
        // The lazy path: `(bytes, [w, h])` produces a TextureInput whose data
        // is `Bytes`, which queues on the Shader's pending list instead of
        // setting the uniform immediately. Renderer::render drains it on the
        // first call.
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            #[rustfmt::skip]
            let pixels: Vec<u8> = vec![
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
            let mat = Material::pbr()
                .expect("pbr")
                .base_color_texture((pixels, [2u32, 2u32]));

            // Before any render: one entry queued on the Shader, no Texture
            // uniform yet (the default 1×1 white texture meta from
            // pbr_defaults is what the bind group will see on the first
            // pass if nobody drains).
            assert_eq!(
                mat.shader.object.pending_textures.read().len(),
                1,
                "expected 1 pending texture before render"
            );

            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat.clone());
            let pass = Pass::new("lazy-upload");
            pass.add(&model).expect("add_model");
            renderer
                .render(&pass, &target)
                .expect("render drains pending");

            // After render: pending list is empty (drained); the Material's
            // shader now carries a real TextureMeta for the base_color slot.
            assert!(
                mat.shader.object.pending_textures.read().is_empty(),
                "expected pending list to be drained by render"
            );
            let img = target.get_image().await;
            assert_eq!(img.len(), 8 * 8 * 4);
        });
    }

    #[test]
    fn texture_setter_with_existing_texture_skips_pending_queue() {
        // The eager path: passing `&Texture` flows through
        // `From<&Texture> for TextureInput` → `TextureData::CloneOf` →
        // immediate uniform write, no pending entry.
        pollster::block_on(async move {
            let renderer = Renderer::new();
            #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
            let tex = renderer
                .create_texture((&pixels[..], crate::Size::from((2u32, 2u32))))
                .await
                .expect("texture");

            let mat = Material::pbr().expect("pbr").base_color_texture(&tex);
            assert!(
                mat.shader.object.pending_textures.read().is_empty(),
                "eager texture path must not queue"
            );
        });
    }
}
