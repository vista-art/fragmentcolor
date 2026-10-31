//! Material — PBR data + `Shader` bundle.
//!
//! `Material::pbr()` ships FragmentColor's default physically-based shader
//! and the glTF 2.0 PBR-MR field set as builder-style setters. The five
//! glTF texture slots (`base_color_map`, `metallic_roughness_map`,
//! `normal_map`, `occlusion_map`, `emissive_map`) start unbound; the
//! renderer fills them with neutral 1×1 fallbacks at draw time when the
//! caller hasn't supplied a texture, so a Material that sets no maps still
//! renders correctly. Each slot samples its own UV set through its own
//! `KHR_texture_transform` (see the `uv` module). `Material::custom(shader)` wraps an arbitrary
//! `Shader` and reuses the same setter API for any uniform the custom
//! shader happens to declare under matching paths.
//!
//! Per-Model transform is *not* a Material uniform — it rides on a Pass-
//! owned per-instance vertex-attribute buffer that the renderer builds from
//! the Model entries `Pass::add` queues. That means many Models can share one Material's
//! Shader without colliding on a `mesh.model` slot; the renderer batches them
//! by pipeline hash and pays one bind-group setup for the whole group.
//! Material itself is `Clone` (shallow, Arc-share) — cloning gives you another
//! handle to the same underlying shader state, not an independent copy.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::texture::{TextureData, TextureMeta, TextureSource};
use crate::{Shader, TextureInput, UniformData};

mod alpha_mode;
pub use alpha_mode::AlphaMode;

mod uv;
pub use uv::{MaterialError, TextureSlot};
pub(crate) use uv::{UvSet, UvTransform};

mod platform;

/// Assembled vertex+fragment+uniform block for the built-in PBR shader. The
/// `mesh/transform` and `material/pbr` snippets are pulled in as composition
/// parts at construction time — they declare no bindings of their own, only
/// helper functions, so they slot in cleanly alongside this main body.
const PBR_MAIN: &str = include_str!("pbr_main.wgsl");

/// The light types and grid constants, which the light binning pass shares.
pub(crate) const LIGHT: &str = include_str!("light.wgsl");

/// The light roots and the loops that read them.
const PBR_LIGHTS: &str = include_str!("pbr_lights.wgsl");

/// The registry snippets the built-in PBR shader composes from, baked in
/// directly rather than resolved through the shader registry. They power the
/// engine's own `Material::pbr()`, so they must be present on every platform
/// regardless of which `shaders-<category>` features the consumer enabled —
/// including the slim web build (`--no-default-features`), whose synchronous
/// constructor cannot fetch them over the network. Embedding the three files
/// (a few KB) keeps `Material::pbr()` infallible and offline everywhere
/// without dragging the rest of the registry into the wasm bundle.
const MESH_TRANSFORM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/mesh/transform.wgsl"
));
const MATERIAL_PBR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/material/pbr.wgsl"
));
const LINEAR_TO_SRGB: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/website/public/shaders/color/linear_to_srgb.wgsl"
));

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
    /// Infallible on every supported build configuration. The snippets the shader
    /// composes from (`mesh/transform`, `material/pbr` and `color/linear_to_srgb`) are baked
    /// into the binary directly, so construction needs no shader registry and
    /// no network — it works the same on native, mobile, and the slim web
    /// build (`--no-default-features`), whose synchronous constructor cannot
    /// fetch over `fetch`.
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr() -> Self {
        // SAFETY-justified expect: the three parts are WGSL source compiled into the binary at build
        // time. The only way this fails is a malformed shader file in the repo, which is a build-time
        // programmer error caught by the doctests and healthchecks, not a runtime fault.
        let shader = Shader::new([
            MESH_TRANSFORM,
            MATERIAL_PBR,
            LINEAR_TO_SRGB,
            LIGHT,
            PBR_LIGHTS,
            PBR_MAIN,
        ])
        .expect("SAFETY: built-in PBR shader parts are compiled-in WGSL source; a parse failure is a build-time error, not a runtime fault");
        let material = Self { shader };
        material.apply_defaults();
        // Seed the ShaderObject. The default ShaderObject double_sided is
        // `true` (preserves the pre-Material no-cull behavior for raw
        // Shader callers); Material::pbr flips it to `false` to follow
        // glTF 2.0's single-sided default.
        material.shader.object.set_alpha_mode(AlphaMode::default());
        material.shader.object.set_double_sided(false);
        material
    }

    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom(shader: Shader) -> Self {
        let mat = Self { shader };
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
        // frame for someone calling `Material::pbr()` and rendering a
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
        // Every texture slot samples UV0 through the identity transform.
        for slot in TextureSlot::ALL {
            self.set_slot_uv(slot, UvSet::First, UvTransform::default());
        }
        let identity = [
            [1.0_f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let _ = self.shader.set("camera.view_proj", identity);
        let _ = self.shader.set("camera.position", [0.0_f32, 0.0, 0.0]);
        // A white light from straight above lights a fresh Material until the
        // first Light reaches it. Ambient is a small gray tint so unlit faces
        // don't read as pitch-black.
        self.shader.object.set_placeholder();
        let _ = self.shader.set("lights.ambient", [0.03_f32, 0.03, 0.03]);
    }

    // --- factor setters ---
    //
    // Every setter takes `&self` and returns `Self` (an Arc-cloned handle
    // to the same Material). This is the same shape Light and Camera use:
    // chains naturally and stays callable on a borrowed Material, so the
    // Python / JavaScript / Swift / Kotlin bindings can mirror the Rust
    // signature exactly and the docs stay one-spelling-fits-all across
    // languages.

    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color(&self, color: [f32; 4]) -> Self {
        set_or_warn(&self.shader, "material.base_color", color);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic(&self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.metallic", value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness(&self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.roughness", value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale(&self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.normal_scale", value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength(&self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.occlusion_strength", value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive(&self, factor: [f32; 3]) -> Self {
        set_or_warn(&self.shader, "material.emissive", factor);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff(&self, value: f32) -> Self {
        set_or_warn(&self.shader, "material.alpha_cutoff", value);
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform(&self, offset: [f32; 2], scale: [f32; 2], rotation: f32) -> Self {
        self.set_transforms(UvTransform {
            offset,
            scale,
            rotation,
        });
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/texture_uv.md")]
    pub fn texture_uv(&self, slot: TextureSlot, set: u32) -> Result<Self, MaterialError> {
        self.set_slot_set(slot, UvSet::try_from(set)?);
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/material/texture_transform.md")]
    pub fn texture_transform(
        &self,
        slot: TextureSlot,
        offset: [f32; 2],
        scale: [f32; 2],
        rotation: f32,
    ) -> Self {
        self.set_slot_transform(
            slot,
            UvTransform {
                offset,
                scale,
                rotation,
            },
        );
        self.clone()
    }

    // --- pipeline-state flags ---

    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode(&self, mode: AlphaMode) -> Self {
        self.shader.object.set_alpha_mode(mode);
        set_or_warn(&self.shader, "material.alpha_mode_flag", mode.flag());
        self.clone()
    }

    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided(&self, value: bool) -> Self {
        self.shader.object.set_double_sided(value);
        self.clone()
    }

    // --- texture setters ---
    //
    // Each setter takes a `Texture`, which binds as it is and stays with its Renderer,
    // or a source (bytes, a path, a URL, a `Mipmap`), which becomes a map: the shader keeps
    // the source, and each Renderer that draws it uploads a copy of its own at `load`.
    //
    // Color-space hints: `base_color` and `emissive` are sRGB per glTF 2.0, and the three
    // maps that hold data (`metallic_roughness`, `normal`, `occlusion`) are linear. A setter
    // picks the slot's format only when the caller left the default; pass a format to override.

    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture(&self, source: impl Into<crate::TextureInput>) -> Self {
        self.set_texture(TextureSlot::BaseColor, source.into())
    }

    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture(&self, source: impl Into<crate::TextureInput>) -> Self {
        self.set_texture(TextureSlot::MetallicRoughness, source.into())
    }

    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture(&self, source: impl Into<crate::TextureInput>) -> Self {
        self.set_texture(TextureSlot::Normal, source.into())
    }

    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture(&self, source: impl Into<crate::TextureInput>) -> Self {
        self.set_texture(TextureSlot::Occlusion, source.into())
    }

    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture(&self, source: impl Into<crate::TextureInput>) -> Self {
        self.set_texture(TextureSlot::Emissive, source.into())
    }
}

pub(crate) fn set_or_warn<V: Into<UniformData>>(shader: &Shader, key: &str, value: V) {
    if let Err(e) = shader.set(key, value) {
        log::warn!("Material setter '{key}' did not apply: {e}");
    }
}

impl Material {
    /// Sets `slot` from `input`. A `Texture` binds as it is, and any other source becomes
    /// a map. An input the slot can't take warns and leaves the slot as it was.
    fn set_texture(&self, slot: TextureSlot, input: TextureInput) -> Self {
        let key = slot.pick_key();
        if let Err(error) = input.check_options(slot.name_setter()) {
            warn_texture(key, error);
            return self.clone();
        }
        if let TextureData::CloneOf(texture) = &input.data {
            let meta = TextureMeta::with_id_only(*texture.id());
            if let Err(error) = self.shader.object.set(key, UniformData::Texture(meta)) {
                warn_texture(key, error);
            }
            return self.clone();
        }
        match TextureSource::try_from(input) {
            Ok(source) => self.set_map(slot, source),
            Err(error) => warn_texture(key, error),
        }
        self.clone()
    }

    /// Sets `slot` to a map of `source`, which takes the slot's format as a hint when it picked none.
    /// A Mipmap or KTX2 container whose bytes don't read in it, such as a float one, keeps its own.
    pub(crate) fn set_map(&self, slot: TextureSlot, mut source: TextureSource) {
        source.take_hint(slot.pick_format());
        let key = slot.pick_key();
        if let Err(error) = self.shader.object.set_map(key, source) {
            warn_texture(key, error);
        }
    }
}

/// Logs a texture setter's input that the Material's `key` couldn't take.
fn warn_texture(key: &str, error: impl std::fmt::Display) {
    log::warn!("Material texture '{key}' did not apply: {error}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{Mesh, Vertex};
    use crate::texture::SourceData;
    use crate::{Pass, Renderer, Target};

    #[test]
    fn pbr_seeds_default_uniforms() {
        let mat = Material::pbr();
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
        let original = Material::pbr().base_color([1.0, 0.0, 0.0, 1.0]);
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
        let mat = Material::pbr().alpha_mode(AlphaMode::Mask);
        // ShaderObject picks up the new mode for the pipeline cache key;
        // the alpha_mode_flag uniform picks it up too so fs_main's Mask
        // discard branch fires.
        assert_eq!(*mat.shader.object.alpha_mode.read(), AlphaMode::Mask);
        let flag: u32 = mat.shader().get("material.alpha_mode_flag").unwrap();
        assert_eq!(flag, AlphaMode::Mask.flag());
    }

    #[test]
    fn double_sided_setter_threads_to_shader_back_reference() {
        let mat = Material::pbr().double_sided(true);
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
            mesh.add_vertex(Vertex::pbr(p).set(Vertex::UV0, uv));
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
            let mat = Material::pbr().base_color([0.6, 0.2, 0.8, 1.0]);
            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat);
            let pass = Pass::new("defaults-only");
            pass.add(&model).expect("add_model");
            renderer
                .render(&pass, &target)
                .expect("render with all default textures");
            let img = target.get_image().await.expect("read back");
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
                .base_color([1.0, 1.0, 1.0, 1.0])
                .base_color_texture(&tex);

            mat.shader()
                .set("camera.position", [0.0_f32, 0.0, 1.0])
                .ok();
            let light = crate::Light::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0]);

            let mesh = Mesh::new();
            for (pos, uv) in [
                ([-1.0_f32, 1.0, 0.0], [0.0_f32, 1.0]),
                ([-1.0, -1.0, 0.0], [0.0, 0.0]),
                ([1.0, -1.0, 0.0], [1.0, 0.0]),
            ] {
                mesh.add_vertex(Vertex::pbr(pos).set(Vertex::UV0, uv));
            }
            let model = crate::scene::Model::new(mesh, mat);
            let pass = Pass::new("textured-triangle");
            pass.add(&model).expect("add_model");
            pass.add(&light).expect("light");
            renderer
                .render(&pass, &target)
                .expect("render textured triangle");
            let img = target.get_image().await.expect("read back");
            assert_eq!(img.len(), 16 * 16 * 4);

            let mut has_red = false;
            let mut has_blue = false;
            for px in img.as_chunks::<4>().0 {
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

    /// The id `material`'s map for `key` holds, the one each Renderer's copy goes under.
    fn read_map(material: &Material, key: &str) -> crate::texture::TextureId {
        let maps = material.shader.object.maps.read();
        let map = maps.iter().find(|map| map.key == key);
        map.map(|map| map.id).expect("a map")
    }

    /// Whether the Renderer that made `target` holds a copy of `material`'s map for `key`.
    fn holds_copy(target: &crate::TextureTarget, material: &Material, key: &str) -> bool {
        let id = read_map(material, key);
        target.texture().context.has_texture(&id)
    }

    /// A Material whose fullscreen shader draws texel (0, 0) of `base_color_map`.
    fn build_mapped() -> Material {
        let shader = crate::Shader::new(
            r#"
@group(0) @binding(0) var base_color_map: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(base_color_map, vec2<i32>(0), 0); }
"#,
        )
        .expect("shader");
        Material::custom(shader)
    }

    /// A 2x2 image of `color`, as raw pixels.
    fn paint(color: [u8; 4]) -> (Vec<u8>, [u32; 2]) {
        (color.repeat(4), [2, 2])
    }

    /// The first pixel of `target`.
    async fn read_pixel(target: &crate::TextureTarget) -> [u8; 4] {
        let image = target.get_image().await.expect("read back");
        [image[0], image[1], image[2], image[3]]
    }

    // Story: raw bytes make a map, which the slot names before any Renderer holds
    // a copy, and the first render uploads a copy into its Renderer under that id.
    #[test]
    fn base_color_texture_with_raw_bytes_uploads_at_render() {
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
            let mat = Material::pbr().base_color_texture((pixels, [2u32, 2u32]));

            let id = read_map(&mat, "base_color_map");
            assert!(
                !holds_copy(&target, &mat, "base_color_map"),
                "a copy before render"
            );

            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat.clone());
            let pass = Pass::new("lazy-upload");
            pass.add(&model).expect("add_model");
            renderer
                .render(&pass, &target)
                .expect("render drains pending");

            assert!(
                holds_copy(&target, &mat, "base_color_map"),
                "no copy after render"
            );
            let slot = mat.shader.object.get_uniform_data("base_color_map");
            assert!(
                matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == id),
                "{slot:?}"
            );
            let img = target.get_image().await.expect("read back");
            assert_eq!(img.len(), 8 * 8 * 4);
        });
    }

    // Story: a map that fails to upload keeps no copy and fails every render the same way,
    // while the other maps upload on the first try.
    #[test]
    fn failed_maps_keep_no_copy_and_the_rest_upload() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");
            let normal: Vec<u8> = [128u8, 128, 255, 255].repeat(4);
            let mat = Material::pbr()
                .base_color_texture("tests/assets/missing.png")
                .normal_texture((normal, [2u32, 2u32]));
            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat.clone());
            let pass = Pass::new("partial-upload");
            pass.add(&model).expect("add_model");

            let first = renderer.render(&pass, &target);
            let second = renderer.render(&pass, &target);

            let first = first.expect_err("the missing map fails the first render");
            let second = second.expect_err("and every render after it");
            assert_eq!(first.to_string(), second.to_string());
            assert!(
                holds_copy(&target, &mat, "normal_map"),
                "the good map uploads on the first render"
            );
            assert!(
                !holds_copy(&target, &mat, "base_color_map"),
                "the failing map keeps no copy"
            );
        });
    }

    // Story: a map the shader doesn't declare warns at the setter and makes no map,
    // so the render draws the maps the shader declares.
    #[test]
    fn an_undeclared_map_warns_at_the_setter() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");
            let pixel: Vec<u8> = [255u8, 0, 0, 255].repeat(4);
            let mat = build_mapped()
                .normal_texture((pixel.clone(), [2u32, 2u32]))
                .base_color_texture((pixel, [2u32, 2u32]));

            assert!(
                !mat.shader.object.has_map("normal_map"),
                "an undeclared map"
            );
            renderer
                .render(mat.shader(), &target)
                .expect("the render draws");
            assert!(holds_copy(&target, &mat, "base_color_map"));
        });
    }

    // Story: the latest setter call for a map wins, so a fixed input replaces a failing one
    // under the same id, and the key keeps one map.
    #[test]
    fn latest_map_setter_wins() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");
            let pixels: Vec<u8> = [255u8, 0, 0, 255].repeat(4);
            let mat = Material::pbr().base_color_texture("tests/assets/missing.png");
            let id = read_map(&mat, "base_color_map");
            let mat = mat.base_color_texture((pixels, [2u32, 2u32]));
            {
                let maps = mat.shader.object.maps.read();
                assert_eq!(maps.len(), 1, "one map for the key");
                assert_eq!(maps[0].id, id, "the map kept its id");
                let latest = matches!(&*maps[0].source.data, SourceData::Bytes(_));
                assert!(latest, "{:?}", maps[0].source);
            }
            let model = crate::scene::Model::new(pbr_triangle_mesh(), mat.clone());
            let pass = Pass::new("replaced-upload");
            pass.add(&model).expect("add_model");

            renderer
                .render(&pass, &target)
                .expect("the replaced map uploads");

            assert!(holds_copy(&target, &mat, "base_color_map"));
        });
    }

    // Story: a texture set while a map's upload fails wins. The load drops the failure
    // of the map that ended and returns Ok, and the render draws the texture.
    #[test]
    fn a_texture_set_during_a_failed_upload_wins() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("a target");
            let texture = renderer
                .create_texture(paint([0, 255, 0, 255]))
                .await
                .expect("a texture");
            let mat = build_mapped().base_color_texture("tests/assets/missing.png");
            let late = mat.clone();
            crate::hooks::once("map uploaded", move || {
                late.base_color_texture(&texture);
            });

            let loaded = renderer.load(mat.shader()).await;

            assert!(
                loaded.is_ok(),
                "the ended map's failure came back: {loaded:?}"
            );
            assert!(
                !mat.shader.object.has_map("base_color_map"),
                "the map stayed"
            );
            renderer.render(mat.shader(), &target).expect("draws");
            assert_eq!(read_pixel(&target).await, [0, 255, 0, 255]);
        });
    }

    // Story: a set that lands while a load uploads the older source wins. The older upload
    // stands in as that Renderer's copy, and each Renderer then draws the latest.
    #[test]
    fn a_set_during_an_upload_ends_on_the_last_value() {
        pollster::block_on(async move {
            let first = Renderer::new();
            let first_target = first.create_texture_target([8, 8]).await.expect("a target");
            let second = Renderer::new();
            let second_target = second
                .create_texture_target([8, 8])
                .await
                .expect("a target");
            let mat = build_mapped().base_color_texture(paint([255, 0, 0, 255]));
            let late = mat.clone();
            crate::hooks::once("map uploaded", move || {
                late.base_color_texture(paint([0, 0, 255, 255]));
            });

            first.load(mat.shader()).await.expect("the load");
            assert!(
                holds_copy(&first_target, &mat, "base_color_map"),
                "the older upload left the first Renderer without a copy"
            );

            first.render(mat.shader(), &first_target).expect("draws");
            second.render(mat.shader(), &second_target).expect("draws");
            assert_eq!(read_pixel(&first_target).await, [0, 0, 255, 255]);
            assert_eq!(read_pixel(&second_target).await, [0, 0, 255, 255]);
        });
    }

    // Story: a `Texture` keeps the options it was created with, so the setter refuses
    // options beside one with a warning, and the slot keeps the texture it held.
    #[test]
    fn a_texture_with_options_leaves_the_slot() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let first = renderer
                .create_texture(paint([255, 0, 0, 255]))
                .await
                .expect("a texture");
            let second = renderer
                .create_texture(paint([0, 0, 255, 255]))
                .await
                .expect("a texture");

            let mat = Material::pbr()
                .base_color_texture(&first)
                .base_color_texture((&second, crate::TextureFormat::Rgba8Unorm));

            let slot = mat.shader.object.get_uniform_data("base_color_map");
            assert!(
                matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == *first.id()),
                "{slot:?}"
            );
        });
    }

    /// A 4x4 KTX2 container of zeros in `format`, at `bytes` per texel.
    fn build_container(format: ktx2::Format, bytes: usize) -> TextureInput {
        use crate::texture::ktx2_loader::test_helpers::Ktx2Test;
        let levels = vec![vec![0u8; 4 * 4 * bytes]];
        let container = Ktx2Test {
            format,
            width: 4,
            height: 4,
            levels,
        };
        TextureInput::from(TextureData::Ktx2Bytes(container.build()))
    }

    // Story: a map takes its slot's format as a hint. A Mipmap or KTX2 container whose bytes
    // read in it uploads in its sRGB twin, and a float one, whose bytes don't, keeps its own.
    #[test]
    fn a_container_map_takes_the_slot_format_only_where_it_reads() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("a target");
            let half = (
                vec![0u8; 4 * 4 * 8],
                crate::TextureFormat::Rgba16Float,
                [4u32, 4u32],
            );
            let half = crate::Mipmap::build(half).expect("a float chain");
            let plain = crate::Mipmap::build((vec![255u8; 4 * 4 * 4], [4u32, 4u32]));
            let plain = plain.expect("a chain");
            let cases: [(TextureInput, wgpu::TextureFormat); 4] = [
                (half.into(), wgpu::TextureFormat::Rgba16Float),
                (plain.into(), wgpu::TextureFormat::Rgba8UnormSrgb),
                (
                    build_container(ktx2::Format::R16G16B16A16_SFLOAT, 8),
                    wgpu::TextureFormat::Rgba16Float,
                ),
                (
                    build_container(ktx2::Format::R8G8B8A8_UNORM, 4),
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                ),
            ];

            for (input, expected) in cases {
                let mat = build_mapped().base_color_texture(input);
                renderer
                    .load(mat.shader())
                    .await
                    .expect("the source uploads");
                let id = read_map(&mat, "base_color_map");
                let copy = target.texture().context.get_texture(&id).expect("a copy");
                assert_eq!(copy.format, expected);
            }
        });
    }

    // Story: a `Texture` binds as it is, so the slot holds its id and no map.
    #[test]
    fn texture_setter_with_existing_texture_makes_no_map() {
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

            let mat = Material::pbr().base_color_texture(&tex);
            assert!(
                mat.shader.object.maps.read().is_empty(),
                "a Texture made a map"
            );
            let slot = mat.shader.object.get_uniform_data("base_color_map");
            assert!(
                matches!(&slot, Ok(UniformData::Texture(meta)) if meta.id == *tex.id()),
                "{slot:?}"
            );
        });
    }

    #[test]
    fn shared_texture_handle_is_reused_across_materials() {
        // One GPU upload, N shader references. The texture ID stored in
        // each Material's `base_color_map` uniform points at the same
        // underlying TextureObject; the renderer's texture registry
        // dedupes by ID at draw time, so N Materials all wallpapered with
        // the same `&shared` texture pay for one GPU resource.
        pollster::block_on(async move {
            let renderer = Renderer::new();
            #[rustfmt::skip]
            let pixels: [u8; 16] = [
                255, 0, 0, 255,    0, 255, 0, 255,
                  0, 0, 255, 255,  255, 255, 255, 255,
            ];
            let shared = renderer
                .create_texture((&pixels[..], crate::Size::from((2u32, 2u32))))
                .await
                .expect("texture");
            let shared_id = *shared.id();

            // 4 Materials, all reading `&shared`. Each Material's
            // `base_color_map` uniform stores the same texture ID.
            let mats: Vec<Material> = (0..4)
                .map(|_| Material::pbr().base_color_texture(&shared))
                .collect();

            for mat in &mats {
                let data = mat
                    .shader()
                    .object
                    .get_uniform_data("base_color_map")
                    .expect("base_color_map");
                match data {
                    crate::UniformData::Texture(meta) => {
                        assert_eq!(
                            meta.id, shared_id,
                            "each Material's base_color_map must point at the shared texture's ID"
                        );
                    }
                    other => panic!("expected UniformData::Texture, got {other:?}"),
                }
            }
        });
    }

    /// Writes `module` back as WGSL, as wgpu does for a browser, and parses that again.
    fn write_for_the_web(module: &naga::Module) -> naga::Module {
        use naga::valid::{Capabilities, ValidationFlags, Validator};
        let info = Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(module)
            .expect("the module validates");
        let flags = naga::back::wgsl::WriterFlags::empty();
        let written = naga::back::wgsl::write_string(module, &info, flags).expect("writes WGSL");
        naga::front::wgsl::parse_str(&written).expect("the written WGSL parses")
    }

    /// The span and member offsets of every struct in `module`, in declaration order.
    /// Names are left out, since the writer renames a struct whose name repeats.
    fn read_layouts(module: &naga::Module) -> Vec<(u32, Vec<u32>)> {
        module
            .types
            .iter()
            .filter_map(|(_, ty)| match &ty.inner {
                naga::TypeInner::Struct { members, span } => {
                    Some((*span, members.iter().map(|member| member.offset).collect()))
                }
                _ => None,
            })
            .collect()
    }

    // Story: on the web wgpu hands the browser the WGSL naga writes back, which drops
    // @size and @align. The PBR shader's uniforms keep their layout through that
    // round trip, on the storage path and on the uniform light path alike.
    #[test]
    fn pbr_layout_survives_the_web_round_trip() {
        // Arrange
        let material = Material::pbr();
        let object = &material.shader().object;
        let uniform = object
            .module_for_device(false, Some(4))
            .expect("the uniform light path lowers");
        let modules = [&object.module, uniform.as_ref()];

        // Act
        let written: Vec<naga::Module> = modules.iter().map(|m| write_for_the_web(m)).collect();

        // Assert
        for (module, written) in modules.iter().zip(&written) {
            let mut validator = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            );
            let checked = validator.validate(written);
            assert!(checked.is_ok(), "the web module fails: {checked:?}");
            assert_eq!(read_layouts(written), read_layouts(module));
        }
    }
}
