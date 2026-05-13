//! Material — PBR data + `Shader` bundle.
//!
//! `Material::pbr()` ships FragmentColor's default physically-based shader
//! and the glTF 2.0 PBR-MR field set as builder-style setters.
//! `Material::custom(shader)` wraps an arbitrary `Shader` and reuses the same
//! setter API for any uniform the custom shader happens to declare under
//! matching paths.
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
    /// that the rest of Material already follows.
    pub(crate) alpha_mode: Arc<RwLock<AlphaMode>>,
    /// Pipeline-state double-sided flag. Same share semantics as
    /// `alpha_mode` above.
    pub(crate) double_sided: Arc<RwLock<bool>>,
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
        let material = Self {
            shader,
            alpha_mode: Arc::new(RwLock::new(AlphaMode::default())),
            double_sided: Arc::new(RwLock::new(false)),
        };
        material.apply_defaults();
        // Seed the back-references on ShaderObject. The default ShaderObject
        // double_sided is `true` (preserves the pre-Material no-cull behaviour
        // for raw Shader callers); Material::pbr flips it to `false` to
        // follow glTF 2.0's single-sided default.
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
        // Mirror the value into the shader's back-reference so the renderer
        // (which iterates `pass.shaders` at draw time, not Materials) can
        // read it when building the RenderPipelineKey.
        self.shader.object.set_alpha_mode(mode);
        // Also reflect the value in the WGSL uniform that gates the Mask
        // discard branch. set_or_warn skips silently on shaders that don't
        // declare `material.alpha_mode_flag` (e.g. Material::custom with a
        // shader that doesn't include the PBR uniform block).
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
    use crate::renderer::renderable::Renderable;
    use crate::target::Target;

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
    fn alpha_mode_setter_updates_shader_back_reference() {
        let mat = Material::pbr().expect("pbr");
        // Default is Opaque on the Material and on the ShaderObject.
        assert_eq!(*mat.alpha_mode.read(), AlphaMode::Opaque);
        assert_eq!(*mat.shader.object.alpha_mode.read(), AlphaMode::Opaque);

        let mat = mat.alpha_mode(AlphaMode::Mask);
        // Both the Material field and the back-reference on ShaderObject
        // see the update. The back-reference is what the renderer reads
        // when building a RenderPipelineKey, so updating only Material
        // without the propagation would silently keep the old pipeline.
        assert_eq!(*mat.alpha_mode.read(), AlphaMode::Mask);
        assert_eq!(*mat.shader.object.alpha_mode.read(), AlphaMode::Mask);

        // The fragment-shader discard branch reads `material.alpha_mode_flag`
        // — verify the uniform mirror also tracks the value.
        let flag: u32 = mat
            .shader()
            .get("material.alpha_mode_flag")
            .expect("alpha_mode_flag uniform");
        assert_eq!(flag, AlphaMode::Mask.flag());
    }

    #[test]
    fn double_sided_setter_updates_shader_back_reference() {
        let mat = Material::pbr().expect("pbr");
        assert!(!*mat.double_sided.read());
        assert!(!*mat.shader.object.double_sided.read());

        let mat = mat.double_sided(true);
        assert!(*mat.double_sided.read());
        assert!(*mat.shader.object.double_sided.read());
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

    // ----- Render smoke tests for AlphaMode and double_sided -----
    //
    // These exercise the renderer end-to-end: write the pipeline-state flags
    // through Material's setters, render to a TextureTarget, then read the
    // pixels back and assert the expected behaviour.

    /// A minimal full-screen triangle shader that exposes the same
    /// `material.alpha_mode_flag` + `material.alpha_cutoff` +
    /// `material.base_color` uniforms the PBR shader uses, so we can drive
    /// the Mask discard branch without standing up a full PBR-MR scene
    /// (camera, lights, per-instance transform, normals, UVs).
    #[cfg(test)]
    fn mask_test_shader_source() -> &'static str {
        r#"
        struct MaskMaterial {
            base_color: vec4<f32>,
            alpha_cutoff: f32,
            alpha_mode_flag: u32,
        }
        @group(0) @binding(0) var<uniform> material: MaskMaterial;

        @vertex
        fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
            // Full-screen triangle (oversized so it covers the viewport in one prim).
            let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
            return vec4<f32>(p[i], 0.0, 1.0);
        }

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            if (material.alpha_mode_flag == 1u && material.base_color.a < material.alpha_cutoff) {
                discard;
            }
            return material.base_color;
        }
        "#
    }

    #[test]
    fn mask_mode_discards_transparent_fragments() {
        use crate::{Pass, Renderer};

        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let shader = Shader::new(mask_test_shader_source()).expect("mask shader");
            let mat = Material::custom(shader)
                // alpha=0.2, cutoff=0.5 -> Mask should discard every fragment.
                .base_color([1.0, 0.2, 0.2, 0.2])
                .alpha_cutoff(0.5)
                .alpha_mode(AlphaMode::Mask);

            let pass = Pass::from_shader("mask-test", mat.shader());
            // Solid green clear so we can see whether anything was discarded.
            pass.passes()
                .first()
                .expect("pass")
                .set_clear_color([0.0, 1.0, 0.0, 1.0]);

            renderer.render(&pass, &target).expect("render ok");

            let img = target.get_image().await;
            // Every fragment was discarded -> the framebuffer keeps its
            // clear-color (RGBA = 0,255,0,255). If the discard didn't fire
            // we'd see the red-ish base_color instead.
            let w = target.size().width as usize;
            let h = target.size().height as usize;
            let i = ((h / 2) * w + (w / 2)) * 4;
            let pixel = [img[i], img[i + 1], img[i + 2], img[i + 3]];
            assert_eq!(
                pixel,
                [0, 255, 0, 255],
                "Mask discard failed; expected clear-green, got {:?}",
                pixel
            );
        });
    }

    #[test]
    fn opaque_mode_keeps_below_cutoff_fragments() {
        // Sanity: in Opaque mode the discard branch is gated off, so the
        // identical material+geometry as the Mask test should produce the
        // red base_color, NOT the clear-green background. Confirms the
        // alpha_mode_flag controls the discard rather than something else
        // happening at the renderer level.
        use crate::{Pass, Renderer};

        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let shader = Shader::new(mask_test_shader_source()).expect("mask shader");
            let mat = Material::custom(shader)
                .base_color([1.0, 0.2, 0.2, 0.2])
                .alpha_cutoff(0.5)
                .alpha_mode(AlphaMode::Opaque);

            let pass = Pass::from_shader("opaque-test", mat.shader());
            pass.passes()
                .first()
                .expect("pass")
                .set_clear_color([0.0, 1.0, 0.0, 1.0]);

            renderer.render(&pass, &target).expect("render ok");

            let img = target.get_image().await;
            let w = target.size().width as usize;
            let h = target.size().height as usize;
            let i = ((h / 2) * w + (w / 2)) * 4;
            let pixel = [img[i], img[i + 1], img[i + 2], img[i + 3]];
            // R channel should be close to 255 (from base_color [1, 0.2, 0.2, 0.2]).
            assert!(
                pixel[0] > 200,
                "Opaque mode should not discard; got pixel {:?}",
                pixel
            );
        });
    }

    /// A flat-color shader that takes a clip-space position straight from
    /// the mesh — no per-instance transform, no camera. Lets the renderer's
    /// `cull_mode` flip determine whether the triangle is visible.
    #[cfg(test)]
    fn flat_color_shader_source() -> &'static str {
        r#"
        @vertex
        fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
            return vec4<f32>(pos, 1.0);
        }

        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
        "#
    }

    #[test]
    fn double_sided_true_renders_back_facing_triangle() {
        use crate::mesh::{Mesh, Vertex};
        use crate::{Pass, Renderer};

        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            // Two vertex orderings of the same triangle: one is front-
            // facing under wgpu's default FrontFace::Ccw, the other is
            // back-facing. We probe both empirically against
            // `double_sided=false` (which culls back faces) to figure out
            // which orientation is the back-facing one, then verify that
            // `double_sided=true` lets that back-facing triangle draw.
            let positions_a = [
                [-0.8_f32, -0.8, 0.0],
                [0.0, 0.8, 0.0],
                [0.8, -0.8, 0.0],
            ];
            let positions_b = [
                [-0.8_f32, -0.8, 0.0],
                [0.8, -0.8, 0.0],
                [0.0, 0.8, 0.0],
            ];

            // Render `positions` with the given `double_sided` setting and
            // return the centre pixel.
            async fn render_once(
                renderer: &Renderer,
                target: &crate::TextureTarget,
                positions: &[[f32; 3]; 3],
                double_sided: bool,
            ) -> [u8; 4] {
                let shader = Shader::new(flat_color_shader_source()).expect("flat shader");
                let mat = Material::custom(shader).double_sided(double_sided);
                let mesh = Mesh::new();
                for v in positions.iter() {
                    mesh.add_vertex(Vertex::new(*v));
                }
                mat.shader().add_mesh(&mesh).expect("attach mesh");
                let pass = Pass::from_shader(
                    if double_sided { "double-sided" } else { "single-sided" },
                    mat.shader(),
                );
                pass.passes()
                    .first()
                    .expect("pass")
                    .set_clear_color([0.0, 1.0, 0.0, 1.0]);
                renderer.render(&pass, target).expect("render");
                let img = target.get_image().await;
                // Sample the exact centre pixel of a 16x16 framebuffer:
                // pixel (8, 8) at byte index (8*16+8)*4 = 544.
                let w = target.size().width as usize;
                let h = target.size().height as usize;
                let cx = w / 2;
                let cy = h / 2;
                let i = (cy * w + cx) * 4;
                [img[i], img[i + 1], img[i + 2], img[i + 3]]
            }

            // First, identify which winding is back-facing under wgpu's
            // default. Render both wound orientations under single-sided
            // (back-face culling on). Whichever produces the green clear
            // colour was back-facing.
            let single_a = render_once(&renderer, &target, &positions_a, false).await;
            let single_b = render_once(&renderer, &target, &positions_b, false).await;
            // Exactly one of the two orientations should have been culled
            // (clear-green pixel). If both drew red or both got culled,
            // something is wrong upstream and the test can't proceed.
            let back_positions = match (single_a, single_b) {
                ([0, 255, 0, 255], px) if px[0] > 200 => &positions_a,
                (px, [0, 255, 0, 255]) if px[0] > 200 => &positions_b,
                other => panic!(
                    "couldn't identify back-facing winding; got single_a={:?}, single_b={:?}",
                    other.0, other.1
                ),
            };

            // Now flip `double_sided=true` on the back-facing triangle.
            // With back-face culling disabled, the red triangle should
            // cover the centre pixel.
            let double_px = render_once(&renderer, &target, back_positions, true).await;
            assert!(
                double_px[0] > 200 && double_px[1] < 80,
                "double-sided: back-facing triangle should now render red; got {:?}",
                double_px
            );
        });
    }
}
