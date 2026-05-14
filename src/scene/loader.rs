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
    /// Build a `SceneSource::Gltf` from anything that converts into a
    /// [`GltfSource`] — `&str` / `&Path` / `PathBuf` for file inputs,
    /// `Vec<u8>` / `&[u8]` for in-memory `.glb` bytes.
    pub fn gltf(source: impl Into<GltfSource>) -> Self {
        Self::Gltf(source.into())
    }
}

/// Payload for `SceneSource::Gltf`. The path variant handles both `.gltf`
/// JSON (with external buffer + image references) and `.glb` binary
/// containers. The bytes variant is `.glb`-only — JSON-with-external-URIs
/// has no anchor for the relative paths to resolve against.
#[derive(Debug, Clone)]
pub enum GltfSource {
    Path(PathBuf),
    /// In-memory bytes of a `.glb` binary container. JSON `.gltf` payloads
    /// with external buffer/image URIs cannot be parsed from raw bytes
    /// alone — load them via a `Path` so the loader can resolve relative
    /// references next to the file.
    Bytes(Vec<u8>),
}

impl From<&str> for GltfSource {
    fn from(s: &str) -> Self {
        GltfSource::Path(PathBuf::from(s))
    }
}
impl From<String> for GltfSource {
    fn from(s: String) -> Self {
        GltfSource::Path(PathBuf::from(s))
    }
}
impl From<&std::path::Path> for GltfSource {
    fn from(p: &std::path::Path) -> Self {
        GltfSource::Path(p.to_path_buf())
    }
}
impl From<PathBuf> for GltfSource {
    fn from(p: PathBuf) -> Self {
        GltfSource::Path(p)
    }
}
impl From<Vec<u8>> for GltfSource {
    fn from(b: Vec<u8>) -> Self {
        GltfSource::Bytes(b)
    }
}
impl From<&[u8]> for GltfSource {
    fn from(b: &[u8]) -> Self {
        GltfSource::Bytes(b.to_vec())
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
    // The gltf crate's `import` resolves external buffers + images from
    // the file's directory; `import_slice` walks bytes alone and only
    // supports `.glb` containers (no external URI resolution).
    let (document, buffers, images) = match source {
        GltfSource::Path(p) => gltf::import(p)?,
        GltfSource::Bytes(b) => gltf::import_slice(&b)?,
    };

    let scene = Scene::new();
    let default_gltf_scene = document
        .default_scene()
        .or_else(|| document.scenes().next());

    if let Some(gltf_scene) = default_gltf_scene {
        for node in gltf_scene.nodes() {
            visit_node(&node, Mat4::IDENTITY, &buffers, &images, &scene)?;
        }
    }

    Ok(scene)
}

/// Depth-first walk over the glTF node tree. Multiplies the node's local
/// transform into the inherited world matrix and dispatches on the node's
/// payload (mesh / camera / KHR_lights_punctual light).
fn visit_node(
    node: &gltf::Node<'_>,
    parent_world: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    scene: &Scene,
) -> Result<(), SceneLoadError> {
    let local = local_transform(node);
    let world = parent_world * local;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let (fc_mesh, fc_material) =
                build_mesh_and_material(&primitive, buffers, images)?;
            let model = Model::new(fc_mesh, fc_material);
            model.set_transform(world.to_cols_array_2d());
            scene.add(&model).map_err(|e| {
                SceneLoadError::Invalid(format!("attaching glTF Model to Scene: {e}"))
            })?;
        }
    }

    if let Some(gltf_camera) = node.camera() {
        let camera = build_camera(&gltf_camera, world);
        scene.add(&camera).map_err(|e| {
            SceneLoadError::Invalid(format!("attaching glTF Camera to Scene: {e}"))
        })?;
    }

    if let Some(gltf_light) = node.light() {
        let light = build_light(&gltf_light, world);
        scene.add(&light).map_err(|e| {
            SceneLoadError::Invalid(format!("attaching glTF Light to Scene: {e}"))
        })?;
    }

    for child in node.children() {
        visit_node(&child, world, buffers, images, scene)?;
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

fn build_mesh_and_material(
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
) -> Result<(Mesh, Material), SceneLoadError> {
    let mesh = Mesh::new();
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| {
            SceneLoadError::Invalid("glTF primitive has no POSITION attribute".into())
        })?
        .collect();
    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|it| it.collect());
    let uvs: Option<Vec<[f32; 2]>> = reader.read_tex_coords(0).map(|it| it.into_f32().collect());

    // glTF allows POSITION-only meshes; the PBR shader expects NORMAL +
    // UV0 too. Fill missing slots with sensible placeholders so unlit /
    // untextured assets still render. Forward-Z normal keeps lighting
    // math finite (a zero normal would NaN out the BRDF); zero UV samples
    // the 1×1 default textures at their corner — neutral and correct.
    // Authoring tools that care about per-vertex shading produce normals;
    // this only matters for hand-built or stripped assets.
    let fallback_normal = [0.0_f32, 0.0, 1.0];
    let fallback_uv = [0.0_f32, 0.0];
    for (i, pos) in positions.iter().enumerate() {
        let n = normals
            .as_ref()
            .and_then(|ns| ns.get(i).copied())
            .unwrap_or(fallback_normal);
        let uv = uvs
            .as_ref()
            .and_then(|us| us.get(i).copied())
            .unwrap_or(fallback_uv);
        mesh.add_vertex(
            crate::mesh::Vertex::new(*pos)
                .set(crate::mesh::Vertex::NORMAL, n)
                .set(crate::mesh::Vertex::UV0, uv),
        );
    }

    if let Some(indices) = reader.read_indices() {
        let idx: Vec<u32> = indices.into_u32().collect();
        mesh.set_indices(idx);
    }

    let material = build_material(&primitive.material(), images)?;
    Ok((mesh, material))
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
        .alpha_mode(map_alpha_mode(gltf_material.alpha_mode()))
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
        material = material.base_color_texture(image_to_texture_input(&info.texture(), images)?);
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        material =
            material.metallic_roughness_texture(image_to_texture_input(&info.texture(), images)?);
    }
    if let Some(info) = gltf_material.normal_texture() {
        material = material.normal_texture(image_to_texture_input(&info.texture(), images)?);
    }
    if let Some(info) = gltf_material.occlusion_texture() {
        material = material.occlusion_texture(image_to_texture_input(&info.texture(), images)?);
    }
    if let Some(info) = gltf_material.emissive_texture() {
        material = material.emissive_texture(image_to_texture_input(&info.texture(), images)?);
    }

    Ok(material)
}

fn map_alpha_mode(mode: gltf::material::AlphaMode) -> crate::material::AlphaMode {
    match mode {
        gltf::material::AlphaMode::Opaque => crate::material::AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => crate::material::AlphaMode::Mask,
        gltf::material::AlphaMode::Blend => crate::material::AlphaMode::Blend,
    }
}

/// Convert a glTF image reference into a TextureInput the lazy Material
/// setter will queue. The gltf crate has already decoded the image bytes,
/// so we wrap them as a `DynamicImage` (no second decode pass) and hand
/// the renderer-side format choice off to the Material's slot hint.
fn image_to_texture_input(
    texture: &gltf::Texture<'_>,
    images: &[gltf::image::Data],
) -> Result<TextureInput, SceneLoadError> {
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
            return Err(SceneLoadError::Invalid(format!(
                "glTF image format {other:?} is not yet supported by the loader"
            )));
        }
    };

    Ok(TextureInput {
        data: TextureData::DynamicImage(dynamic),
        options: Default::default(),
    })
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
    let (_, rotation, translation) = decompose_trs(world);
    let eye: [f32; 3] = translation.into();
    let forward = rotation * Vec3::new(0.0, 0.0, -1.0);
    let up = rotation * Vec3::new(0.0, 1.0, 0.0);
    let target_v = translation + forward;
    camera.look_at(eye, target_v.into(), up.into())
}

/// Lift a `KHR_lights_punctual` node into a FragmentColor [`Light`]. The
/// world matrix gives the position (point/spot) and the rotated `-Z`
/// direction (directional/spot).
fn build_light(gltf_light: &gltf::khr_lights_punctual::Light<'_>, world: Mat4) -> Light {
    let color = gltf_light.color();
    let intensity = gltf_light.intensity();
    let range = gltf_light.range().unwrap_or(0.0);
    let (_, rotation, translation) = decompose_trs(world);
    // glTF spec: lights look down `-Z` in their local frame.
    let direction: [f32; 3] = (rotation * Vec3::new(0.0, 0.0, -1.0)).into();
    let position: [f32; 3] = translation.into();

    match gltf_light.kind() {
        gltf::khr_lights_punctual::Kind::Directional => Light::directional(direction, color)
            .set_intensity(intensity),
        gltf::khr_lights_punctual::Kind::Point => Light::point(position, color)
            .set_intensity(intensity)
            .set_range(range),
        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => Light::spot(position, direction, color)
            .set_intensity(intensity)
            .set_range(range)
            .set_cone_angles(inner_cone_angle, outer_cone_angle),
    }
}

fn decompose_trs(m: Mat4) -> (Vec3, Quat, Vec3) {
    let (scale, rotation, translation) = m.to_scale_rotation_translation();
    (scale, rotation, translation)
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
        while json_bytes.len() % 4 != 0 {
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
}
