//! Scene loader — `Scene::load` and the `SceneSource` transport.
//!
//! The loader is format-tagged: every entry into `Scene::load` carries a
//! [`SceneSource`] variant that names which file format the bytes-or-path
//! payload should be parsed as. glTF 2.0 (with its `.glb` binary container)
//! is the first format; the enum exists so future additions land without a
//! breaking change to the public method.
//!
//! The shape mirrors `Material::pbr()` and `Camera::perspective(...)`: sync
//! return, no `Renderer` argument. Any texture inputs the parser produces
//! flow into the Material as pending [`crate::TextureInput`] entries and
//! the renderer drains them on first render (see [`crate::Renderer::load`]).
//!
//! Today the loader covers static glTF: mesh primitives (positions,
//! normals, UVs, indices), PBR-MR materials with all five texture slots,
//! per-node transforms flattened into Model matrices, glTF camera nodes,
//! and `KHR_lights_punctual` lights. Animation, skinning, morph targets,
//! and material extensions beyond PBR-MR are out of scope here.

use std::path::PathBuf;

use glam::{Mat4, Quat, Vec3};
use lsp_doc::lsp_doc;

use crate::scene::{Camera, Light, Scene};
use crate::{Material, Mesh, Model, TextureData, TextureInput};

/// Top-level loader input. Every variant names the format explicitly so
/// `Scene::load` doesn't have to guess. Today: glTF 2.0; future additions
/// (USD, FBX, …) slot in as new variants.
#[derive(Debug, Clone)]
pub enum SceneSource {
    Gltf(GltfSource),
}

impl SceneSource {
    /// Build a [`GltfSource`] from anything that converts into one — `&str`
    /// / `&Path` / `PathBuf` for file inputs, `Vec<u8>` / `&[u8]` for
    /// in-memory `.glb` bytes. Chains naturally with the filter methods on
    /// `GltfSource` (`cameras`, `lights`, …); the resulting `GltfSource`
    /// converts to `SceneSource` automatically when handed to
    /// [`Scene::load`](crate::Scene::load), so `Scene::load(SceneSource::gltf(path))`
    /// keeps working unchanged.
    pub fn gltf(source: impl Into<GltfSource>) -> GltfSource {
        source.into()
    }
}

/// Payload for `SceneSource::Gltf`. Carries the file-or-bytes input plus
/// per-load filter options that decide which glTF node kinds the loader
/// instantiates as FragmentColor objects.
///
/// Built via the `From` impls below — `&str` / `String` / `&Path` / `PathBuf`
/// for file inputs, `Vec<u8>` / `&[u8]` for in-memory `.glb` bytes. Filter
/// methods chain after the conversion: `SceneSource::gltf("model.glb")
/// .cameras(false).lights(false)` skips embedded camera + light nodes
/// during the walk, leaving the user free to attach their own.
#[derive(Debug, Clone)]
pub struct GltfSource {
    payload: GltfPayload,
    options: GltfOptions,
}

#[derive(Debug, Clone)]
pub(crate) enum GltfPayload {
    Path(PathBuf),
    /// In-memory bytes of a `.glb` binary container. JSON `.gltf` payloads
    /// with external buffer/image URIs cannot be parsed from raw bytes
    /// alone — load them via a `Path` so the loader can resolve relative
    /// references next to the file.
    Bytes(Vec<u8>),
}

/// Per-load loader filters. Adding a new filter means one extra `bool` here
/// plus one extra check in `visit_node`. Stay additive — every filter has
/// `true` as the inclusive default so the no-arg path keeps loading
/// everything the loader is capable of.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GltfOptions {
    pub(crate) cameras: bool,
    pub(crate) lights: bool,
}

impl Default for GltfOptions {
    fn default() -> Self {
        Self {
            cameras: true,
            lights: true,
        }
    }
}

impl GltfSource {
    fn from_payload(payload: GltfPayload) -> Self {
        Self {
            payload,
            options: GltfOptions::default(),
        }
    }

    /// Toggle glTF camera-node instantiation. Default `true`; pass `false`
    /// when the consumer brings its own [`Camera`](crate::Camera) and the
    /// embedded camera would just fight for the same shader uniforms.
    #[lsp_doc("docs/api/scene/gltf_source/cameras.md")]
    pub fn cameras(mut self, on: bool) -> Self {
        self.options.cameras = on;
        self
    }

    /// Toggle `KHR_lights_punctual` light-node instantiation. Default `true`;
    /// pass `false` when the consumer brings its own [`Light`](crate::Light)
    /// rig (cursor lighting, animated key/fill, …) and the embedded lights
    /// would compete for the same `lights.lights[..]` slots.
    #[lsp_doc("docs/api/scene/gltf_source/lights.md")]
    pub fn lights(mut self, on: bool) -> Self {
        self.options.lights = on;
        self
    }

    pub(crate) fn into_parts(self) -> (GltfPayload, GltfOptions) {
        (self.payload, self.options)
    }
}

impl From<&str> for GltfSource {
    fn from(s: &str) -> Self {
        Self::from_payload(GltfPayload::Path(PathBuf::from(s)))
    }
}
impl From<String> for GltfSource {
    fn from(s: String) -> Self {
        Self::from_payload(GltfPayload::Path(PathBuf::from(s)))
    }
}
impl From<&std::path::Path> for GltfSource {
    fn from(p: &std::path::Path) -> Self {
        Self::from_payload(GltfPayload::Path(p.to_path_buf()))
    }
}
impl From<PathBuf> for GltfSource {
    fn from(p: PathBuf) -> Self {
        Self::from_payload(GltfPayload::Path(p))
    }
}
impl From<Vec<u8>> for GltfSource {
    fn from(b: Vec<u8>) -> Self {
        Self::from_payload(GltfPayload::Bytes(b))
    }
}
impl From<&[u8]> for GltfSource {
    fn from(b: &[u8]) -> Self {
        Self::from_payload(GltfPayload::Bytes(b.to_vec()))
    }
}

impl From<GltfSource> for SceneSource {
    fn from(g: GltfSource) -> Self {
        SceneSource::Gltf(g)
    }
}

/// Errors from `Scene::load`. Wraps the upstream parser's typed error
/// surface where we can.
#[derive(Debug, thiserror::Error)]
pub enum SceneLoadError {
    #[error("glTF parse error: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("PBR material construction failed: {0}")]
    Material(#[from] crate::ShaderError),
    #[error("invalid glTF data: {0}")]
    Invalid(String),
}

/// Entry point invoked by `Scene::load`. Dispatches on the format tag.
pub(crate) fn load(source: SceneSource) -> Result<Scene, SceneLoadError> {
    match source {
        SceneSource::Gltf(gltf) => load_gltf(gltf),
    }
}

fn load_gltf(source: GltfSource) -> Result<Scene, SceneLoadError> {
    let (payload, options) = source.into_parts();
    // The gltf crate's `import` resolves external buffers + images from
    // the file's directory; `import_slice` walks bytes alone and only
    // supports `.glb` containers (no external URI resolution). The Path
    // variant goes through `std::fs`, which doesn't exist on the
    // wasm32-unknown-unknown target — wasm callers must hand the bytes
    // themselves (typically via `fetch` + `Uint8Array`).
    let (document, buffers, images) = match payload {
        #[cfg(not(wasm))]
        GltfPayload::Path(p) => gltf::import(p)?,
        #[cfg(wasm)]
        GltfPayload::Path(_) => {
            return Err(SceneLoadError::Invalid(
                "Scene::load(SceneSource::gltf(path)) is not supported on wasm32 — fetch the bytes from JS and pass them via SceneSource::gltf(bytes)".into(),
            ));
        }
        GltfPayload::Bytes(b) => gltf::import_slice(&b)?,
    };

    let scene = Scene::new();
    let default_gltf_scene = document
        .default_scene()
        .or_else(|| document.scenes().next());

    // glTF files routinely share one Material across many primitives. The
    // cache key is the glTF material index (`None` is the spec's default
    // material) so a file with 100 indexed primitives produces one
    // Material handle instead of 100 shader allocations.
    let mut material_cache: std::collections::HashMap<Option<usize>, Material> =
        std::collections::HashMap::new();

    if let Some(gltf_scene) = default_gltf_scene {
        for node in gltf_scene.nodes() {
            visit_node(
                &node,
                Mat4::IDENTITY,
                &buffers,
                &images,
                &scene,
                &mut material_cache,
                &options,
            )?;
        }
    }

    Ok(scene)
}

/// Depth-first walk over the glTF node tree. Multiplies the node's local
/// transform into the inherited world matrix and dispatches on the node's
/// payload (mesh / camera / KHR_lights_punctual light). Filter toggles in
/// [`GltfOptions`] short-circuit the camera and light branches when the
/// caller opted out.
fn visit_node(
    node: &gltf::Node<'_>,
    parent_world: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    scene: &Scene,
    material_cache: &mut std::collections::HashMap<Option<usize>, Material>,
    options: &GltfOptions,
) -> Result<(), SceneLoadError> {
    let local = local_transform(node);
    let world = parent_world * local;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let fc_mesh = build_mesh(&primitive, buffers)?;
            let mat_key = primitive.material().index();
            let fc_material = match material_cache.get(&mat_key) {
                Some(m) => m.clone(),
                None => {
                    let m = build_material(&primitive.material(), images)?;
                    material_cache.insert(mat_key, m.clone());
                    m
                }
            };
            let model = Model::new(fc_mesh, fc_material);
            model.set_transform(world.to_cols_array_2d());
            scene.add(&model).map_err(|e| {
                SceneLoadError::Invalid(format!("attaching glTF Model to Scene: {e}"))
            })?;
        }
    }

    if options.cameras
        && let Some(gltf_camera) = node.camera()
    {
        let camera = build_camera(&gltf_camera, world);
        scene.add(&camera).map_err(|e| {
            SceneLoadError::Invalid(format!("attaching glTF Camera to Scene: {e}"))
        })?;
    }

    if options.lights
        && let Some(gltf_light) = node.light()
    {
        add_light(&gltf_light, world, scene)?;
    }

    for child in node.children() {
        visit_node(&child, world, buffers, images, scene, material_cache, options)?;
    }
    Ok(())
}

/// glTF nodes carry either `matrix` or `(translation, rotation, scale)`;
/// the `transform()` helper hides the disjunction and gives us a typed
/// shape we can lift straight into glam's right-handed convention.
fn local_transform(node: &gltf::Node<'_>) -> Mat4 {
    match node.transform() {
        gltf::scene::Transform::Matrix { matrix } => Mat4::from_cols_array_2d(&matrix),
        gltf::scene::Transform::Decomposed {
            translation,
            rotation,
            scale,
        } => Mat4::from_scale_rotation_translation(
            Vec3::from(scale),
            Quat::from_array(rotation),
            Vec3::from(translation),
        ),
    }
}

fn build_mesh(
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<Mesh, SceneLoadError> {
    let mesh = Mesh::new();
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| {
            SceneLoadError::Invalid("glTF primitive has no POSITION attribute".into())
        })?
        .collect();
    let supplied_normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|it| it.collect());
    let uvs: Option<Vec<[f32; 2]>> = reader.read_tex_coords(0).map(|it| it.into_f32().collect());
    let uv1s: Option<Vec<[f32; 2]>> = reader.read_tex_coords(1).map(|it| it.into_f32().collect());
    let colors: Option<Vec<[f32; 4]>> =
        reader.read_colors(0).map(|it| it.into_rgba_f32().collect());
    let tangents: Option<Vec<[f32; 4]>> = reader.read_tangents().map(|it| it.collect());
    let indices: Option<Vec<u32>> = reader.read_indices().map(|it| it.into_u32().collect());

    // glTF normals are optional; when missing, compute per-vertex normals
    // by accumulating face normals across every triangle that touches the
    // vertex (area-weighted average from the un-normalized cross product).
    // Produces smooth shading on closed meshes and matches what authoring
    // tools would have written. Falls back to the +Z placeholder only for
    // degenerate vertices that touch no triangle.
    let computed_normals = if supplied_normals.is_none() {
        Some(compute_vertex_normals(&positions, indices.as_deref()))
    } else {
        None
    };
    let normals = supplied_normals.as_ref().or(computed_normals.as_ref());

    let fallback_normal = [0.0_f32, 0.0, 1.0];
    let fallback_uv = [0.0_f32, 0.0];
    let fallback_color = [1.0_f32, 1.0, 1.0, 1.0];
    // glTF spec: tangent.xyz is unit-length, .w is ±1 (bitangent sign).
    // The default `[1, 0, 0, 1]` gives a sensible T = +X / B = +Y / N = +Z
    // basis for assets that don't ship tangents; normal-map sampling on
    // surfaces that face +Z stays correct, while non-Z surfaces will look
    // wrong until the loader computes MikkTSpace tangents (follow-up).
    let fallback_tangent = [1.0_f32, 0.0, 0.0, 1.0];
    for (i, pos) in positions.iter().enumerate() {
        let n = normals
            .and_then(|ns| ns.get(i).copied())
            .unwrap_or(fallback_normal);
        let uv = uvs
            .as_ref()
            .and_then(|us| us.get(i).copied())
            .unwrap_or(fallback_uv);
        let uv1 = uv1s
            .as_ref()
            .and_then(|us| us.get(i).copied())
            .unwrap_or(fallback_uv);
        let color = colors
            .as_ref()
            .and_then(|cs| cs.get(i).copied())
            .unwrap_or(fallback_color);
        let tangent = tangents
            .as_ref()
            .and_then(|ts| ts.get(i).copied())
            .unwrap_or(fallback_tangent);
        mesh.add_vertex(
            crate::mesh::Vertex::new(*pos)
                .set(crate::mesh::Vertex::NORMAL, n)
                .set(crate::mesh::Vertex::UV0, uv)
                .set(crate::mesh::Vertex::COLOR0, color)
                .set(crate::mesh::Vertex::UV1, uv1)
                .set(crate::mesh::Vertex::TANGENT, tangent),
        );
    }

    if let Some(idx) = indices {
        mesh.set_indices(idx);
    }

    Ok(mesh)
}

/// Smooth per-vertex normals from positions + an optional index buffer.
/// Accumulates the un-normalized cross product of each triangle into the
/// vertex slots it touches (area-weighted contribution by construction);
/// the final normalize step turns the accumulated direction into a unit
/// vector. Degenerate vertices (zero accumulated direction, or untouched
/// by any triangle) fall back to a forward-Z normal so the BRDF stays
/// finite — matches the glTF 2.0 default for POSITION-only meshes.
fn compute_vertex_normals(positions: &[[f32; 3]], indices: Option<&[u32]>) -> Vec<[f32; 3]> {
    let mut accum = vec![Vec3::ZERO; positions.len()];
    let n_verts = positions.len();
    let visit_triangle = |a: usize, b: usize, c: usize, accum: &mut [Vec3]| {
        if a >= n_verts || b >= n_verts || c >= n_verts {
            return;
        }
        let v0 = Vec3::from(positions[a]);
        let v1 = Vec3::from(positions[b]);
        let v2 = Vec3::from(positions[c]);
        let face = (v1 - v0).cross(v2 - v0);
        accum[a] += face;
        accum[b] += face;
        accum[c] += face;
    };
    if let Some(idx) = indices {
        for tri in idx.chunks_exact(3) {
            visit_triangle(tri[0] as usize, tri[1] as usize, tri[2] as usize, &mut accum);
        }
    } else {
        let mut i = 0;
        while i + 2 < n_verts {
            visit_triangle(i, i + 1, i + 2, &mut accum);
            i += 3;
        }
    }
    accum
        .iter()
        .map(|v| {
            v.try_normalize()
                .unwrap_or(Vec3::new(0.0, 0.0, 1.0))
                .to_array()
        })
        .collect()
}

/// Translate a glTF texture's sampler into FragmentColor's
/// [`SamplerOptions`]. glTF's `MIRRORED_REPEAT` collapses to `REPEAT`
/// (FragmentColor's sampler doesn't expose mirror today); mipmap-filter
/// variants of `min_filter` collapse to their base filter — the upload
/// path runs its own mipmap chain decision based on `options.mipmaps`.
fn map_sampler_options(sampler: &gltf::texture::Sampler<'_>) -> crate::SamplerOptions {
    use gltf::texture::{MagFilter, MinFilter, WrappingMode};
    let smooth = match (sampler.mag_filter(), sampler.min_filter()) {
        // `Nearest` mag-filter is the strongest signal for pixel-art /
        // texel-art assets; respect it. Linear (or unspecified) keeps
        // the FragmentColor default smooth=true.
        (Some(MagFilter::Nearest), _) => false,
        (_, Some(MinFilter::Nearest))
        | (_, Some(MinFilter::NearestMipmapNearest))
        | (_, Some(MinFilter::NearestMipmapLinear)) => false,
        _ => true,
    };
    let repeat_x = matches!(
        sampler.wrap_s(),
        WrappingMode::Repeat | WrappingMode::MirroredRepeat
    );
    let repeat_y = matches!(
        sampler.wrap_t(),
        WrappingMode::Repeat | WrappingMode::MirroredRepeat
    );
    crate::SamplerOptions {
        repeat_x,
        repeat_y,
        smooth,
        compare: None,
    }
}

fn build_material(
    gltf_material: &gltf::Material<'_>,
    images: &[gltf::image::Data],
) -> Result<Material, SceneLoadError> {
    let pbr = gltf_material.pbr_metallic_roughness();
    let mut material = Material::pbr()?
        .base_color(pbr.base_color_factor())
        .metallic(pbr.metallic_factor())
        .roughness(pbr.roughness_factor())
        .emissive(gltf_material.emissive_factor())
        .alpha_cutoff(gltf_material.alpha_cutoff().unwrap_or(0.5))
        .alpha_mode(match gltf_material.alpha_mode() {
            gltf::material::AlphaMode::Opaque => crate::material::AlphaMode::Opaque,
            gltf::material::AlphaMode::Mask => crate::material::AlphaMode::Mask,
            gltf::material::AlphaMode::Blend => crate::material::AlphaMode::Blend,
        })
        .double_sided(gltf_material.double_sided());

    if let Some(scale) = gltf_material
        .normal_texture()
        .map(|info| info.scale())
    {
        material = material.normal_scale(scale);
    }
    if let Some(strength) = gltf_material
        .occlusion_texture()
        .map(|info| info.strength())
    {
        material = material.occlusion_strength(strength);
    }

    if let Some(info) = pbr.base_color_texture() {
        if let Some(input) = try_image_to_texture_input(&info.texture(), images, "base_color")? {
            material = material.base_color_texture(input);
        }
        // KHR_texture_transform on the base-color slot is the most common
        // usage of the extension; promote it to the Material's global
        // transform. The non-base-color slots warn-and-ignore — see the
        // `warn_unused_*` helpers below.
        if let Some(t) = info.texture_transform() {
            let offset = t.offset();
            let scale = t.scale();
            let rotation = t.rotation();
            material = material.uv_transform(offset, scale, rotation);
        }
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        warn_unused_uv_transform(&info, "metallic_roughness");
        if let Some(input) =
            try_image_to_texture_input(&info.texture(), images, "metallic_roughness")?
        {
            material = material.metallic_roughness_texture(input);
        }
    }
    if let Some(info) = gltf_material.normal_texture()
        && let Some(input) = try_image_to_texture_input(&info.texture(), images, "normal")?
    {
        material = material.normal_texture(input);
    }
    if let Some(info) = gltf_material.occlusion_texture()
        && let Some(input) = try_image_to_texture_input(&info.texture(), images, "occlusion")?
    {
        material = material.occlusion_texture(input);
    }
    if let Some(info) = gltf_material.emissive_texture() {
        warn_unused_uv_transform(&info, "emissive");
        if let Some(input) = try_image_to_texture_input(&info.texture(), images, "emissive")? {
            material = material.emissive_texture(input);
        }
    }

    Ok(material)
}

/// Warn when a non-base-color glTF texture carries a
/// `KHR_texture_transform` payload that the current PBR shader can't
/// apply. Only `base_color` plumbs through to the Material's global
/// `uv_transform`; the other four slots silently use the identity
/// transform, so a glTF file that ships per-map transforms on those
/// slots will render at the wrong UVs without this warning. The gltf
/// crate's `NormalTexture` / `OcclusionTexture` don't expose
/// `texture_transform()` so the helper only covers the three slots
/// reached through `texture::Info` (metallic_roughness, emissive,
/// base_color — base_color itself reads its transform separately).
fn warn_unused_uv_transform(info: &gltf::texture::Info<'_>, slot: &str) {
    if info.texture_transform().is_some() {
        log::warn!(
            "Scene::load ignoring KHR_texture_transform on `{slot}` slot — only `base_color` honours per-map transforms today"
        );
    }
}

/// Convert a glTF image reference into a TextureInput the lazy Material
/// setter will queue. Returns `Ok(None)` for unsupported pixel formats so
/// the caller can skip that slot and the Material falls back to its 1×1
/// default — losing the whole `Scene::load` to one stray 16-bit-per-channel
/// PNG was the prior behaviour and it's worse than silently degraded
/// shading on a single map. Hard errors (missing image, mismatched byte
/// counts) still bubble up.
fn try_image_to_texture_input(
    texture: &gltf::Texture<'_>,
    images: &[gltf::image::Data],
    slot: &str,
) -> Result<Option<TextureInput>, SceneLoadError> {
    let image = images
        .get(texture.source().index())
        .ok_or_else(|| SceneLoadError::Invalid("glTF texture references missing image".into()))?;

    let buffer = image.pixels.clone();
    let dynamic = match image.format {
        gltf::image::Format::R8G8B8A8 => {
            image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(image.width, image.height, buffer).ok_or_else(|| {
                    SceneLoadError::Invalid(
                        "glTF RGBA image has fewer bytes than width × height × 4".into(),
                    )
                })?,
            )
        }
        gltf::image::Format::R8G8B8 => {
            image::DynamicImage::ImageRgb8(
                image::RgbImage::from_raw(image.width, image.height, buffer).ok_or_else(|| {
                    SceneLoadError::Invalid(
                        "glTF RGB image has fewer bytes than width × height × 3".into(),
                    )
                })?,
            )
        }
        gltf::image::Format::R8 => {
            image::DynamicImage::ImageLuma8(
                image::GrayImage::from_raw(image.width, image.height, buffer).ok_or_else(|| {
                    SceneLoadError::Invalid(
                        "glTF luminance image has fewer bytes than width × height".into(),
                    )
                })?,
            )
        }
        other => {
            log::warn!(
                "Scene::load skipping `{slot}` slot: glTF image format {other:?} is not yet supported by the loader; falling back to the Material's 1×1 default for this slot"
            );
            return Ok(None);
        }
    };

    let sampler_options = map_sampler_options(&texture.sampler());
    Ok(Some(TextureInput {
        data: TextureData::DynamicImage(dynamic),
        options: crate::TextureOptions {
            sampler: sampler_options,
            ..Default::default()
        },
    }))
}

/// Lift a glTF camera node into a FragmentColor [`Camera`]. The node's
/// world matrix gives the eye position + view orientation; the camera's
/// projection comes from the glTF camera projection block.
fn build_camera(gltf_camera: &gltf::Camera<'_>, world: Mat4) -> Camera {
    let camera = match gltf_camera.projection() {
        gltf::camera::Projection::Perspective(p) => {
            // glTF's aspectRatio is optional; fall back to 1.0 (a common
            // default that gets overridden when the user retargets to a
            // specific render target).
            let aspect = p.aspect_ratio().unwrap_or(1.0);
            // glTF's zfar is optional too; map "infinite" to a large
            // finite value the depth buffer can still represent.
            let far = p.zfar().unwrap_or(1.0e6);
            Camera::perspective(p.yfov(), aspect, p.znear(), far)
        }
        gltf::camera::Projection::Orthographic(o) => {
            let half_w = o.xmag();
            let half_h = o.ymag();
            Camera::orthographic(-half_w, half_w, -half_h, half_h, o.znear(), o.zfar())
        }
    };

    // Derive eye + target + up from the world matrix. glTF nodes that
    // hold a camera look down `-Z` with `+Y` up by convention, so we
    // transform those local axes by the world matrix.
    let (_, rotation, translation) = world.to_scale_rotation_translation();
    let eye: [f32; 3] = translation.into();
    let forward = rotation * Vec3::new(0.0, 0.0, -1.0);
    let up = rotation * Vec3::new(0.0, 1.0, 0.0);
    let target_v = translation + forward;
    camera.look_at(eye, target_v.into(), up.into())
}

/// Lift a `KHR_lights_punctual` node into the unified [`Light`] type and
/// attach it to the Scene. The world matrix gives the position
/// (point/spot) and the rotated `-Z` direction (directional/spot). The
/// kind-specific setters (`set_range`, `set_cone_angles`) are called only
/// on the kinds that accept them — directional gets the universal
/// `set_intensity` only.
fn add_light(
    gltf_light: &gltf::khr_lights_punctual::Light<'_>,
    world: Mat4,
    scene: &Scene,
) -> Result<(), SceneLoadError> {
    let color = gltf_light.color();
    let intensity = gltf_light.intensity();
    let range = gltf_light.range().unwrap_or(0.0).max(0.0);
    let (_, rotation, translation) = world.to_scale_rotation_translation();
    // glTF spec: lights look down `-Z` in their local frame.
    let direction: [f32; 3] = (rotation * Vec3::new(0.0, 0.0, -1.0)).into();
    let position: [f32; 3] = translation.into();

    let attach_err =
        |e: crate::PassError| SceneLoadError::Invalid(format!("attaching glTF Light to Scene: {e}"));
    let setter_err = |e: crate::scene::LightError| {
        SceneLoadError::Invalid(format!("configuring glTF Light: {e}"))
    };

    let light = match gltf_light.kind() {
        gltf::khr_lights_punctual::Kind::Directional => {
            Light::directional(direction, color).set_intensity(intensity)
        }
        gltf::khr_lights_punctual::Kind::Point => {
            let l = Light::point(position, color).set_intensity(intensity);
            l.set_range(range).map_err(setter_err)?
        }
        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => {
            let l = Light::spot(position, direction, color).set_intensity(intensity);
            let l = l.set_range(range).map_err(setter_err)?;
            l.set_cone_angles(inner_cone_angle, outer_cone_angle)
                .map_err(setter_err)?
        }
    };
    scene.add(&light).map_err(attach_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Renderable;

    /// Build a minimal valid .glb in memory: one triangle, positions only.
    /// Three vertices (vec3 floats) packed into the BIN chunk; the JSON
    /// chunk wires them up through a single buffer / bufferView /
    /// accessor / mesh primitive / node / scene.
    fn build_minimal_triangle_glb() -> Vec<u8> {
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let bin_len = bin.len() as u32;

        let json = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;
        let mut json_bytes = json.as_bytes().to_vec();
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }
        let json_len = json_bytes.len() as u32;
        let total = 12 + 8 + json_len + 8 + bin_len;

        let mut glb = Vec::with_capacity(total as usize);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total.to_le_bytes());
        glb.extend_from_slice(&json_len.to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&bin_len.to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&bin);
        glb
    }

    /// Same triangle as `build_minimal_triangle_glb` but with one extra
    /// node that holds a perspective camera, and one directional light
    /// declared via `KHR_lights_punctual`. Gives the filter tests a real
    /// camera + light to drop.
    fn build_triangle_with_camera_and_light_glb() -> Vec<u8> {
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let bin_len = bin.len() as u32;

        // Scene with three nodes: mesh (0), camera (1), light (2).
        // glTF camera uses perspective; KHR_lights_punctual adds one
        // directional light at extension index 0.
        // (Kept on one line — `scripts/no_panics.rs` does a line-by-line
        // brace scan that doesn't track multi-line raw strings, and an
        // un-tracked raw string in test code throws off the test-region
        // detection elsewhere in this file.)
        let json = r#"{"asset":{"version":"2.0"},"extensionsUsed":["KHR_lights_punctual"],"extensions":{"KHR_lights_punctual":{"lights":[{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}]}},"scene":0,"scenes":[{"nodes":[0,1,2]}],"nodes":[{"mesh":0},{"camera":0},{"extensions":{"KHR_lights_punctual":{"light":0}}}],"cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}]}"#;
        let mut json_bytes = json.as_bytes().to_vec();
        while !json_bytes.len().is_multiple_of(4) {
            json_bytes.push(b' ');
        }
        let json_len = json_bytes.len() as u32;
        let total = 12 + 8 + json_len + 8 + bin_len;

        let mut glb = Vec::with_capacity(total as usize);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total.to_le_bytes());
        glb.extend_from_slice(&json_len.to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&bin_len.to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&bin);
        glb
    }

    #[test]
    fn load_default_options_keeps_gltf_camera_and_light() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(SceneSource::gltf(bytes)).expect("load");
        assert_eq!(scene.models().len(), 1, "geometry stays");
        assert_eq!(scene.cameras().len(), 1, "default options load the embedded camera");
        assert_eq!(scene.lights().len(), 1, "default options load the embedded light");
    }

    #[test]
    fn cameras_filter_skips_gltf_camera_nodes() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(SceneSource::gltf(bytes).cameras(false)).expect("load");
        assert_eq!(scene.models().len(), 1, "geometry stays");
        assert!(scene.cameras().is_empty(), "cameras filter drops the camera node");
        assert_eq!(scene.lights().len(), 1, "lights still load by default");
    }

    #[test]
    fn lights_filter_skips_khr_lights_punctual() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(SceneSource::gltf(bytes).lights(false)).expect("load");
        assert_eq!(scene.models().len(), 1);
        assert_eq!(scene.cameras().len(), 1, "cameras still load by default");
        assert!(scene.lights().is_empty(), "lights filter drops the light node");
    }

    #[test]
    fn both_filters_off_yields_geometry_only_scene() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(
            SceneSource::gltf(bytes).cameras(false).lights(false),
        )
        .expect("load");
        assert_eq!(scene.models().len(), 1, "geometry survives every filter");
        assert!(scene.cameras().is_empty());
        assert!(scene.lights().is_empty());
    }

    #[test]
    fn load_minimal_triangle_glb_returns_scene_with_one_model() {
        let bytes = build_minimal_triangle_glb();
        let scene = Scene::load(SceneSource::gltf(bytes)).expect("load triangle.glb");
        let passes = scene.passes();
        assert_eq!(passes.len(), 1, "expected one default pass");
        // The Scene's default pass should hold one Model entry (one
        // primitive in the test asset).
        assert_eq!(
            passes[0].model_entries.read().len(),
            1,
            "expected one Model on the default pass"
        );
    }

    #[test]
    fn load_falls_back_through_into_for_path_inputs() {
        // Just exercises the From<&str> -> GltfSource -> SceneSource chain;
        // the actual file IO error is what we want to see.
        let result = Scene::load(SceneSource::gltf("/definitely/not/a/real/path.glb"));
        assert!(result.is_err(), "expected a load error for a bogus path");
    }

    #[test]
    fn compute_vertex_normals_indexed_yz_face() {
        // Triangle in the YZ plane. Face normal = +X. Without
        // face-normal computation the fallback +Z would shade this
        // triangle as if it pointed forward — visibly wrong.
        let positions = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let indices = [0u32, 1, 2];
        let normals = compute_vertex_normals(&positions, Some(&indices));
        for n in normals {
            assert!((n[0] - 1.0).abs() < 1.0e-6, "got {n:?}");
            assert!(n[1].abs() < 1.0e-6);
            assert!(n[2].abs() < 1.0e-6);
        }
    }

    #[test]
    fn compute_vertex_normals_non_indexed_walks_triplets() {
        // Same YZ-plane triangle but unindexed: positions in sequential
        // triplets. The loader treats every three consecutive positions
        // as one face.
        let positions = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let normals = compute_vertex_normals(&positions, None);
        for n in normals {
            assert!((n[0] - 1.0).abs() < 1.0e-6, "got {n:?}");
        }
    }

    #[test]
    fn compute_vertex_normals_averages_shared_vertex() {
        // Two coplanar triangles sharing vertex 0 — both contribute the
        // same +Z face normal, so the shared vertex normalizes to +Z too
        // (un-degenerate after normalization).
        let positions = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let indices = [0u32, 1, 2, 0, 3, 4];
        let normals = compute_vertex_normals(&positions, Some(&indices));
        assert!((normals[0][2] - 1.0).abs() < 1.0e-6, "shared {normals:?}");
    }

    #[test]
    fn map_sampler_options_handles_repeat_and_nearest() {
        // Build a minimal glTF programmatically that includes a sampler
        // with REPEAT wrap + NEAREST filtering, then exercise the
        // translation to FragmentColor's SamplerOptions.
        let json = r#"{
            "asset": { "version": "2.0" },
            "scene": 0,
            "scenes": [{ "nodes": [] }],
            "samplers": [{
                "magFilter": 9728,
                "minFilter": 9728,
                "wrapS": 10497,
                "wrapT": 10497
            }]
        }"#;
        let doc = gltf::Gltf::from_slice(json.as_bytes())
            .expect("parse glTF JSON")
            .document;
        let sampler = doc.samplers().next().expect("sampler");
        let opts = map_sampler_options(&sampler);
        assert!(opts.repeat_x);
        assert!(opts.repeat_y);
        assert!(!opts.smooth, "magFilter=9728 (NEAREST) should set smooth=false");
    }
}
