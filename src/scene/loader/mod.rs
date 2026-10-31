//! Scene loader: `Scene::load` and the `SceneSource` transport.
//!
//! `Scene::load` accepts a path or in-memory bytes directly. The input
//! becomes a [`SceneSource`] tagged with its format, so the loader knows
//! which parser reads it. glTF 2.0, with its `.glb` binary container, is the one format.
//!
//! The shape mirrors `Material::pbr()` and `Camera::perspective(...)`: sync
//! return, no `Renderer` argument. Each image stays encoded, as its file's bytes or
//! its path, and every slot that reads it becomes a Material map of that one source.
//! Each Renderer that draws the Scene decodes it into a copy of its own (see [`crate::Renderer::load`]).
//!
//! The loader covers static glTF: mesh primitives (positions, normals,
//! two UV sets, colors, tangents, indices), PBR-MR materials with all five
//! texture slots, each with the UV set and `KHR_texture_transform` it names,
//! per-node transforms flattened into Model matrices, glTF camera nodes,
//! and `KHR_lights_punctual` lights. Animation, skinning, morph targets,
//! and material extensions beyond PBR-MR are out of scope here.

use std::collections::{HashMap, HashSet};
#[cfg(not(wasm))]
use std::path::Path;
use std::path::PathBuf;

use glam::{Mat4, Quat, Vec3};

use crate::core::shader::error::describe;
use crate::scene::{Camera, Light, Scene};
use crate::texture::TextureSource;
use crate::{Material, Model};

mod images;
use images::read_images;
mod materials;
use materials::build_material;
mod meshes;
use meshes::build_mesh;

/// The loader's input, tagged with its format.
/// glTF 2.0 is its one variant, so `Scene::load` never guesses the format.
#[derive(Debug, Clone)]
pub enum SceneSource {
    Gltf(GltfSource),
}

/// Payload for `SceneSource::Gltf`. Carries the file-or-bytes input plus
/// per-load filter options that decide which glTF node kinds the loader
/// instantiates as FragmentColor objects.
///
/// Built via the `From` impls below — `&str` / `String` / `&Path` / `PathBuf`
/// for file inputs, `Vec<u8>` / `&[u8]` for in-memory `.glb` bytes. Filter
/// methods chain after the conversion: `GltfSource::from("model.glb")
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
    pub fn cameras(mut self, on: bool) -> Self {
        self.options.cameras = on;
        self
    }

    /// Toggle `KHR_lights_punctual` light-node instantiation. Default `true`;
    /// pass `false` when the consumer brings its own [`Light`](crate::Light)
    /// rig (cursor lighting, animated key/fill, …) and the embedded lights
    /// would only add to it.
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

// Direct conversions so `Scene::load(path)` / `Scene::load(bytes)` work
// without the caller naming a format. Every raw input is glTF today; when a
// second format arrives, branch here on the file extension (path inputs) or
// a magic-byte sniff (byte inputs) to pick the variant.
impl From<&str> for SceneSource {
    fn from(s: &str) -> Self {
        SceneSource::Gltf(GltfSource::from(s))
    }
}
impl From<String> for SceneSource {
    fn from(s: String) -> Self {
        SceneSource::Gltf(GltfSource::from(s))
    }
}
impl From<&std::path::Path> for SceneSource {
    fn from(p: &std::path::Path) -> Self {
        SceneSource::Gltf(GltfSource::from(p))
    }
}
impl From<PathBuf> for SceneSource {
    fn from(p: PathBuf) -> Self {
        SceneSource::Gltf(GltfSource::from(p))
    }
}
impl From<Vec<u8>> for SceneSource {
    fn from(b: Vec<u8>) -> Self {
        SceneSource::Gltf(GltfSource::from(b))
    }
}
impl From<&[u8]> for SceneSource {
    fn from(b: &[u8]) -> Self {
        SceneSource::Gltf(GltfSource::from(b))
    }
}

/// Errors from `Scene::load`.
#[derive(Debug, thiserror::Error)]
pub enum SceneLoadError {
    #[error("glTF parse error: {0}")]
    Gltf(String),
    #[error("PBR material construction failed: {0}")]
    Material(#[from] crate::ShaderError),
    #[error("invalid glTF data: {0}")]
    Invalid(String),
    #[error("glTF camera {index}: {source}. Load with cameras(false) to skip the file's cameras")]
    Camera {
        index: usize,
        source: crate::scene::CameraError,
    },
}

/// A failure of the glTF parser, which `SceneLoadError::Gltf` carries as text.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(crate) struct GltfFailure(#[from] gltf::Error);

impl From<GltfFailure> for SceneLoadError {
    fn from(failure: GltfFailure) -> Self {
        SceneLoadError::Gltf(describe(&failure.0))
    }
}

/// What the glTF parser reads from a file: its document, its buffers, and the folder
/// its external files sit in when it came from a path. Images stay unread here.
struct GltfParts {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    base: Option<PathBuf>,
}

/// The parts of a glTF file the parser reads from `path`, with the buffers beside it.
#[cfg(not(wasm))]
fn import_path(path: &Path) -> Result<GltfParts, GltfFailure> {
    let base = path
        .parent()
        .unwrap_or_else(|| Path::new("./"))
        .to_path_buf();
    let gltf::Gltf { document, blob } = gltf::Gltf::open(path)?;
    let buffers = gltf::import_buffers(&document, Some(&base), blob)?;
    Ok(GltfParts {
        document,
        buffers,
        base: Some(base),
    })
}

/// The parts of a `.glb` or embedded glTF file the parser reads from `bytes`.
fn import_bytes(bytes: &[u8]) -> Result<GltfParts, GltfFailure> {
    let gltf::Gltf { document, blob } = gltf::Gltf::from_slice(bytes)?;
    let buffers = gltf::import_buffers(&document, None, blob)?;
    Ok(GltfParts {
        document,
        buffers,
        base: None,
    })
}

/// Entry point invoked by `Scene::load`. Dispatches on the format tag.
pub(crate) fn load(source: SceneSource) -> Result<Scene, SceneLoadError> {
    match source {
        SceneSource::Gltf(gltf) => load_gltf(gltf),
    }
}

fn load_gltf(source: GltfSource) -> Result<Scene, SceneLoadError> {
    let (payload, options) = source.into_parts();
    // A path resolves external buffers and images from the file's folder, and bytes
    // reach only what the file holds. A path goes through `std::fs`, which wasm32
    // lacks, so wasm callers fetch the bytes themselves and pass them.
    let parts = match payload {
        #[cfg(not(wasm))]
        GltfPayload::Path(p) => import_path(&p)?,
        #[cfg(wasm)]
        GltfPayload::Path(path) => {
            return Err(SceneLoadError::Invalid(format!(
                "Scene::load can't read the path {} on wasm32. Fetch the bytes from JS and pass them to Scene::load(bytes)",
                path.display()
            )));
        }
        GltfPayload::Bytes(b) => import_bytes(&b)?,
    };
    let GltfParts {
        document,
        buffers,
        base,
    } = parts;
    let images = read_images(&document, &buffers, base.as_deref())?;

    let scene = Scene::new();
    let default_gltf_scene = document
        .default_scene()
        .or_else(|| document.scenes().next());

    let mut cache = Cache::default();

    if let Some(gltf_scene) = default_gltf_scene {
        // An explicit stack walks the tree depth first in file order. A deep
        // chain of nodes can't overflow the call stack, and `visited` refuses
        // a node reached twice: glTF nodes form disjoint trees, so a second
        // visit means a cycle or a node with two parents.
        let mut stack: Vec<(gltf::Node<'_>, Mat4)> = gltf_scene
            .nodes()
            .map(|node| (node, Mat4::IDENTITY))
            .collect();
        stack.reverse();
        let mut visited = HashSet::new();
        while let Some((node, parent_world)) = stack.pop() {
            if !visited.insert(node.index()) {
                return Err(SceneLoadError::Invalid(format!(
                    "glTF node {} is reached twice; nodes must form trees, without cycles or shared children",
                    node.index()
                )));
            }
            let world = visit_node(
                &node,
                parent_world,
                &buffers,
                &images,
                &scene,
                &mut cache,
                &options,
            )?;
            let first_child = stack.len();
            stack.extend(node.children().map(|child| (child, world)));
            stack[first_child..].reverse();
        }
    }

    Ok(scene)
}

/// What one load builds once and shares while it walks the nodes.
#[derive(Default)]
struct Cache {
    /// One Material per glTF material index (`None` is the spec's default material),
    /// since files routinely share one Material across many primitives.
    materials: HashMap<Option<usize>, Material>,
}

/// Load one glTF node and return its world matrix for its children.
///
/// Multiplies the node's local transform into the inherited world matrix
/// and dispatches on the node's payload (mesh / camera / KHR_lights_punctual
/// light). Filter toggles in [`GltfOptions`] short-circuit the camera and
/// light branches when the caller opted out.
fn visit_node(
    node: &gltf::Node<'_>,
    parent_world: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[TextureSource],
    scene: &Scene,
    cache: &mut Cache,
    options: &GltfOptions,
) -> Result<Mat4, SceneLoadError> {
    let local = local_transform(node);
    let world = parent_world * local;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let mesh = build_mesh(&mesh, &primitive, buffers)?;
            let key = primitive.material().index();
            let material = match cache.materials.get(&key) {
                Some(material) => material.clone(),
                None => {
                    let material = build_material(&primitive.material(), images)?;
                    cache.materials.insert(key, material.clone());
                    material
                }
            };
            let model = Model::new(mesh, material);
            model.set_transform(world.to_cols_array_2d());
            scene.add(&model).map_err(|error| {
                SceneLoadError::Invalid(format!("attaching glTF Model to Scene: {error}"))
            })?;
        }
    }

    if options.cameras
        && let Some(gltf_camera) = node.camera()
    {
        let camera = build_camera(&gltf_camera, world)?;
        scene.add(&camera).map_err(|error| {
            SceneLoadError::Invalid(format!("attaching glTF Camera to Scene: {error}"))
        })?;
    }

    if options.lights
        && let Some(gltf_light) = node.light()
    {
        add_light(&gltf_light, world, scene)?;
    }

    Ok(world)
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

/// Lift a glTF camera node into a FragmentColor [`Camera`].
///
/// The node's world matrix gives the eye position and view orientation, and
/// the camera's projection block gives the projection. A perspective camera
/// without `zfar` asks for an infinite projection, and it gets one. Errors
/// from `Camera` carry the camera's index inside `SceneLoadError::Camera`.
fn build_camera(gltf_camera: &gltf::Camera<'_>, world: Mat4) -> Result<Camera, SceneLoadError> {
    let index = gltf_camera.index();
    let blame_camera = |source| SceneLoadError::Camera { index, source };
    let (near, zfar) = match gltf_camera.projection() {
        gltf::camera::Projection::Perspective(perspective) => {
            (perspective.znear(), perspective.zfar())
        }
        gltf::camera::Projection::Orthographic(orthographic) => {
            (orthographic.znear(), Some(orthographic.zfar()))
        }
    };
    // A Camera takes far < near as reverse-Z, but glTF requires zfar > znear.
    if let Some(far) = zfar
        && far <= near
    {
        return Err(SceneLoadError::Invalid(format!(
            "glTF camera {index} has zfar {far} at or before znear {near}"
        )));
    }
    let camera = match gltf_camera.projection() {
        gltf::camera::Projection::Perspective(perspective) => {
            // glTF's aspectRatio is optional; fall back to 1.0 (a common
            // default that gets overridden when the user retargets to a
            // specific render target).
            let aspect = perspective.aspect_ratio().unwrap_or(1.0);
            match zfar {
                Some(far) => Camera::perspective(perspective.yfov(), aspect, near, far),
                None => Camera::create_infinite(perspective.yfov(), aspect, near),
            }
            .map_err(blame_camera)?
        }
        gltf::camera::Projection::Orthographic(orthographic) => {
            let half_width = orthographic.xmag();
            let half_height = orthographic.ymag();
            let far = orthographic.zfar();
            Camera::orthographic(
                -half_width,
                half_width,
                -half_height,
                half_height,
                near,
                far,
            )
            .map_err(blame_camera)?
        }
    };

    // Derive eye + target + up from the world matrix. glTF nodes that
    // hold a camera look down `-Z` with `+Y` up by convention, so we
    // transform those local axes by the world matrix.
    let (_, rotation, translation) = world.to_scale_rotation_translation();
    let eye: [f32; 3] = translation.into();
    let forward = rotation * Vec3::new(0.0, 0.0, -1.0);
    let up = rotation * Vec3::new(0.0, 1.0, 0.0);
    let target = translation + forward;
    camera
        .look_at(eye, target.into(), up.into())
        .map_err(blame_camera)
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
    let range = gltf_light.range().unwrap_or(0.0);
    let (_, rotation, translation) = world.to_scale_rotation_translation();
    // glTF spec: lights look down `-Z` in their local frame.
    let direction: [f32; 3] = (rotation * Vec3::new(0.0, 0.0, -1.0)).into();
    let position: [f32; 3] = translation.into();

    let attach_err = |e: crate::PassError| {
        SceneLoadError::Invalid(format!("attaching glTF Light to Scene: {e}"))
    };
    let setter_err = |e: crate::scene::LightError| {
        SceneLoadError::Invalid(format!(
            "configuring glTF light {}: {e}",
            gltf_light.index()
        ))
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

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for SceneLoadError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::SceneLoad
    }
}

#[cfg(test)]
mod tests;
