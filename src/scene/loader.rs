//! Scene loader: `Scene::load` and the `SceneSource` transport.
//!
//! `Scene::load` accepts a path or in-memory bytes directly. The input
//! becomes a [`SceneSource`] tagged with its format, so the loader knows
//! which parser reads it. glTF 2.0, with its `.glb` binary container, is the one format.
//!
//! The shape mirrors `Material::pbr()` and `Camera::perspective(...)`: sync
//! return, no `Renderer` argument. Each image decodes once, and every slot
//! that reads it becomes a Material map of those shared pixels, which each
//! Renderer that draws the Scene uploads as a copy of its own (see [`crate::Renderer::load`]).
//!
//! The loader covers static glTF: mesh primitives (positions, normals,
//! two UV sets, colors, tangents, indices), PBR-MR materials with all five
//! texture slots, each with the UV set and `KHR_texture_transform` it names,
//! per-node transforms flattened into Model matrices, glTF camera nodes,
//! and `KHR_lights_punctual` lights. Animation, skinning, morph targets,
//! and material extensions beyond PBR-MR are out of scope here.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use gltf::Semantic;
use gltf::accessor::{DataType, Dimensions};

use crate::material::{TextureSlot, UvSet, UvTransform};
use crate::scene::{Camera, Light, Scene};
use crate::texture::{SourceData, TextureSource};
use crate::{Material, Mesh, Model, TextureOptions};

/// The loader's input, tagged with its format.
/// glTF 2.0 is its one variant, so `Scene::load` never guesses the format.
#[derive(Debug, Clone)]
pub enum SceneSource {
    Gltf(GltfSource),
}

impl SceneSource {
    /// Build a [`GltfSource`] from anything that converts into one — `&str`
    /// / `&Path` / `PathBuf` for file inputs, `Vec<u8>` / `&[u8]` for
    /// in-memory `.glb` bytes. Use this when you need the per-load filter
    /// methods (`cameras`, `lights`); the resulting `GltfSource` converts to
    /// `SceneSource` automatically when handed to
    /// [`Scene::load`](crate::Scene::load). For the common case, pass the
    /// path or bytes straight to `Scene::load` instead.
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
    /// A file the glTF parser refuses, with its reason.
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
        SceneLoadError::Gltf(crate::shader::error::describe(&failure.0))
    }
}

/// What the glTF parser reads from a file: its document, buffers and images.
type GltfParts = (
    gltf::Document,
    Vec<gltf::buffer::Data>,
    Vec<gltf::image::Data>,
);

/// The parts of a glTF file the parser reads from `path`, with the files beside it.
#[cfg(not(wasm))]
fn import_path(path: &std::path::Path) -> Result<GltfParts, GltfFailure> {
    Ok(gltf::import(path)?)
}

/// The parts of a `.glb` or embedded glTF file the parser reads from `bytes`.
fn import_bytes(bytes: &[u8]) -> Result<GltfParts, GltfFailure> {
    Ok(gltf::import_slice(bytes)?)
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
    /// The decoded pixels of each image by index. Every slot that reads one image,
    /// in any material, shares its one buffer.
    pixels: HashMap<usize, Arc<SourceData>>,
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
    images: &[gltf::image::Data],
    scene: &Scene,
    cache: &mut Cache,
    options: &GltfOptions,
) -> Result<Mat4, SceneLoadError> {
    let local = local_transform(node);
    let world = parent_world * local;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let fc_mesh = build_mesh(&mesh, &primitive, buffers)?;
            let mat_key = primitive.material().index();
            let fc_material = match cache.materials.get(&mat_key) {
                Some(m) => m.clone(),
                None => {
                    let m = build_material(&primitive.material(), images, &mut cache.pixels)?;
                    cache.materials.insert(mat_key, m.clone());
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
        let camera = build_camera(&gltf_camera, world)?;
        scene
            .add(&camera)
            .map_err(|e| SceneLoadError::Invalid(format!("attaching glTF Camera to Scene: {e}")))?;
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

/// Accessor types the loader reads for each semantic.
const VEC3_FLOAT: &[(DataType, Dimensions)] = &[(DataType::F32, Dimensions::Vec3)];
const VEC4_FLOAT: &[(DataType, Dimensions)] = &[(DataType::F32, Dimensions::Vec4)];
const UV_TYPES: &[(DataType, Dimensions)] = &[
    (DataType::U8, Dimensions::Vec2),
    (DataType::U16, Dimensions::Vec2),
    (DataType::F32, Dimensions::Vec2),
];
const COLOR_TYPES: &[(DataType, Dimensions)] = &[
    (DataType::U8, Dimensions::Vec3),
    (DataType::U16, Dimensions::Vec3),
    (DataType::F32, Dimensions::Vec3),
    (DataType::U8, Dimensions::Vec4),
    (DataType::U16, Dimensions::Vec4),
    (DataType::F32, Dimensions::Vec4),
];
const INDEX_TYPES: &[(DataType, Dimensions)] = &[
    (DataType::U8, Dimensions::Scalar),
    (DataType::U16, Dimensions::Scalar),
    (DataType::U32, Dimensions::Scalar),
];

/// Check every accessor of a primitive before the gltf reader runs.
///
/// The reader panics on a component type its semantic doesn't allow, on
/// a stride shorter than one element and on a count of 0, so the loader
/// refuses those here. Each attribute it reads must also hold one element
/// per position. Attributes the loader never reads only need a count.
fn check_primitive(
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<(), SceneLoadError> {
    let position_count = primitive
        .get(&Semantic::Positions)
        .map(|accessor| accessor.count());
    for (semantic, accessor) in primitive.attributes() {
        let accepted = match semantic {
            Semantic::Positions | Semantic::Normals => VEC3_FLOAT,
            Semantic::Tangents => VEC4_FLOAT,
            Semantic::TexCoords(0 | 1) => UV_TYPES,
            Semantic::Colors(0) => COLOR_TYPES,
            _ => &[],
        };
        let name = semantic.to_string();
        check_accessor(&accessor, &name, accepted, buffers)?;
        if let Some(positions) = position_count
            && !accepted.is_empty()
            && accessor.count() != positions
        {
            return Err(SceneLoadError::Invalid(format!(
                "glTF accessor {} ({name}) holds {} elements for {positions} positions",
                accessor.index(),
                accessor.count()
            )));
        }
    }
    let file_bytes = buffers.iter().map(|data| data.len() as u64).sum();
    if let Some(positions) = primitive.get(&Semantic::Positions) {
        check_backing(&positions, "POSITION", file_bytes)?;
    }
    if let Some(indices) = primitive.indices() {
        check_accessor(&indices, "indices", INDEX_TYPES, buffers)?;
        check_backing(&indices, "indices", file_bytes)?;
    }
    Ok(())
}

/// Refuse a vertex or index count the file's bytes don't back.
///
/// With a bufferView the count is bounded by the view. A sparse accessor
/// without one stores only its changed values, so it may span no more bytes
/// than all of the file's buffers hold. Every other attribute the loader reads
/// matches the vertex count, so memory grows with the file and not with a number.
fn check_backing(
    accessor: &gltf::Accessor<'_>,
    semantic: &str,
    file_bytes: u64,
) -> Result<(), SceneLoadError> {
    if accessor.view().is_some() {
        return Ok(());
    }
    // check_accessor has capped the count at u32::MAX, so this can't overflow.
    let bytes = accessor.count() as u64 * accessor.size() as u64;
    if bytes > file_bytes {
        return Err(SceneLoadError::Invalid(format!(
            "glTF accessor {} ({semantic}) spans {bytes} bytes without a bufferView, more than the {file_bytes} bytes the file's buffers hold",
            accessor.index()
        )));
    }
    Ok(())
}

/// Check one accessor's type, count and byte range.
/// An empty `accepted` list marks an attribute the loader never reads, so
/// only its count is checked. The arithmetic is checked too, because a huge
/// count would otherwise overflow inside the reader.
fn check_accessor(
    accessor: &gltf::Accessor<'_>,
    semantic: &str,
    accepted: &[(DataType, Dimensions)],
    buffers: &[gltf::buffer::Data],
) -> Result<(), SceneLoadError> {
    let label = format!("glTF accessor {} ({semantic})", accessor.index());
    let count = accessor.count();
    if count == 0 {
        return Err(SceneLoadError::Invalid(format!(
            "{label} has count 0; glTF requires at least 1"
        )));
    }
    if accepted.is_empty() {
        return Ok(());
    }
    let layout = (accessor.data_type(), accessor.dimensions());
    if !accepted.contains(&layout) {
        return Err(SceneLoadError::Invalid(format!(
            "{label} holds {:?} {:?}, a type {semantic} doesn't take",
            layout.0, layout.1
        )));
    }
    if count > u32::MAX as usize {
        return Err(SceneLoadError::Invalid(format!(
            "{label} holds {count} elements, more than 32-bit indices reach"
        )));
    }
    let size = accessor.size();
    let Some(sparse) = accessor.sparse() else {
        let view = accessor.view().ok_or_else(|| {
            SceneLoadError::Invalid(format!("{label} has neither a bufferView nor sparse data"))
        })?;
        return check_range(&label, &view, accessor.offset(), count, size, buffers);
    };
    if sparse.count() == 0 {
        return Err(SceneLoadError::Invalid(format!(
            "{label} has a sparse count of 0; glTF requires at least 1"
        )));
    }
    if let Some(view) = accessor.view() {
        check_range(&label, &view, accessor.offset(), count, size, buffers)?;
    }
    let indices = sparse.indices();
    let index_size = indices.index_type().size();
    check_range(
        &label,
        &indices.view(),
        indices.offset(),
        sparse.count(),
        index_size,
        buffers,
    )?;
    let values = sparse.values();
    check_range(
        &label,
        &values.view(),
        values.offset(),
        sparse.count(),
        size,
        buffers,
    )
}

/// Check that `count` elements of `size` bytes fit their buffer view.
/// The view's stride must hold one element, the last element must end inside
/// the view, and the view must end inside its buffer. Every sum is checked.
fn check_range(
    label: &str,
    view: &gltf::buffer::View<'_>,
    offset: usize,
    count: usize,
    size: usize,
    buffers: &[gltf::buffer::Data],
) -> Result<(), SceneLoadError> {
    let stride = view.stride().unwrap_or(size);
    if stride < size {
        return Err(SceneLoadError::Invalid(format!(
            "{label} reads {size}-byte elements from bufferView {} with byteStride {stride}",
            view.index()
        )));
    }
    let end = count
        .checked_sub(1)
        .and_then(|last| stride.checked_mul(last))
        .and_then(|span| span.checked_add(offset))
        .and_then(|start| start.checked_add(size));
    if end.is_none_or(|end| end > view.length()) {
        return Err(SceneLoadError::Invalid(format!(
            "{label} reads past the end of bufferView {}",
            view.index()
        )));
    }
    let view_end = view.offset().checked_add(view.length());
    let buffer_length = buffers.get(view.buffer().index()).map(|data| data.len());
    match (view_end, buffer_length) {
        (Some(end), Some(length)) if end <= length => Ok(()),
        _ => Err(SceneLoadError::Invalid(format!(
            "{label} reads bufferView {}, which runs past the end of buffer {}",
            view.index(),
            view.buffer().index()
        ))),
    }
}

/// Reserve room for `count` items, or name what the allocator refused.
/// An overcommitting system may grant more than it can fill, which is why
/// `check_backing` first ties every vertex and index count to the file's bytes.
fn reserve<Item>(count: usize, what: &str) -> Result<Vec<Item>, SceneLoadError> {
    let mut items = Vec::new();
    if items.try_reserve_exact(count).is_err() {
        return Err(SceneLoadError::Invalid(format!(
            "{what} needs {count} elements, more than the allocator gives"
        )));
    }
    Ok(items)
}

/// Collect a declared accessor's items, or `None` for an absent one.
/// The reader returns `None` both for an absent attribute and for one whose
/// bytes it can't reach, and only the first case may fall back to defaults.
fn read_items<Item>(
    declared: Option<gltf::Accessor<'_>>,
    items: Option<impl Iterator<Item = Item>>,
    semantic: &str,
) -> Result<Option<Vec<Item>>, SceneLoadError> {
    let (accessor, items) = match (declared, items) {
        (None, _) => return Ok(None),
        (Some(accessor), None) => {
            return Err(SceneLoadError::Invalid(format!(
                "glTF accessor {} ({semantic}) can't be read from its buffer",
                accessor.index()
            )));
        }
        (Some(accessor), Some(items)) => (accessor, items),
    };
    let label = format!("glTF accessor {} ({semantic})", accessor.index());
    let mut values = reserve(accessor.count(), &label)?;
    values.extend(items);
    Ok(Some(values))
}

fn build_mesh(
    gltf_mesh: &gltf::Mesh<'_>,
    primitive: &gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<Mesh, SceneLoadError> {
    // Errors that name no accessor name the primitive, so a large file can be searched.
    let label = format!(
        "glTF mesh {} primitive {}",
        gltf_mesh.index(),
        primitive.index()
    );
    check_primitive(primitive, buffers)?;
    let mesh = Mesh::new();
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &data[..]));

    let positions: Vec<[f32; 3]> = read_items(
        primitive.get(&Semantic::Positions),
        reader.read_positions(),
        "POSITION",
    )?
    .ok_or_else(|| SceneLoadError::Invalid(format!("{label} has no POSITION attribute")))?;
    let supplied_normals: Option<Vec<[f32; 3]>> = read_items(
        primitive.get(&Semantic::Normals),
        reader.read_normals(),
        "NORMAL",
    )?;
    let uvs: Option<Vec<[f32; 2]>> = read_items(
        primitive.get(&Semantic::TexCoords(0)),
        reader.read_tex_coords(0).map(|it| it.into_f32()),
        "TEXCOORD_0",
    )?;
    let uv1s: Option<Vec<[f32; 2]>> = read_items(
        primitive.get(&Semantic::TexCoords(1)),
        reader.read_tex_coords(1).map(|it| it.into_f32()),
        "TEXCOORD_1",
    )?;
    let colors: Option<Vec<[f32; 4]>> = read_items(
        primitive.get(&Semantic::Colors(0)),
        reader.read_colors(0).map(|it| it.into_rgba_f32()),
        "COLOR_0",
    )?;
    let tangents: Option<Vec<[f32; 4]>> = read_items(
        primitive.get(&Semantic::Tangents),
        reader.read_tangents(),
        "TANGENT",
    )?;
    let indices: Option<Vec<u32>> = read_items(
        primitive.indices(),
        reader.read_indices().map(|it| it.into_u32()),
        "indices",
    )?;
    let indices = list_triangles(&label, primitive.mode(), indices, positions.len())?;
    check_indices(&label, indices.as_deref(), positions.len())?;

    // glTF normals are optional; when missing, compute per-vertex normals
    // by accumulating face normals across every triangle that touches the
    // vertex (area-weighted average from the un-normalized cross product).
    // Produces smooth shading on closed meshes and matches what authoring
    // tools would have written. Falls back to the +Z placeholder only for
    // degenerate vertices that touch no triangle.
    let computed_normals = if supplied_normals.is_none() {
        Some(compute_vertex_normals(
            &label,
            &positions,
            indices.as_deref(),
        )?)
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
        // A primitive without TEXCOORD_1 gives its UV0 to the slots on set 1.
        let uv1 = uv1s
            .as_ref()
            .and_then(|us| us.get(i).copied())
            .unwrap_or(uv);
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

/// Turn a primitive's draw mode into triangle list indices.
/// A triangle list keeps its indices. A strip or a fan expands its index
/// list, or every vertex in order when it has none, into one triangle per
/// step. Points and lines have no triangles, so the loader refuses them.
fn list_triangles(
    label: &str,
    mode: gltf::mesh::Mode,
    indices: Option<Vec<u32>>,
    vertex_count: usize,
) -> Result<Option<Vec<u32>>, SceneLoadError> {
    use gltf::mesh::Mode;
    let expand = match mode {
        Mode::Triangles => return Ok(indices),
        Mode::TriangleStrip => expand_strip,
        Mode::TriangleFan => expand_fan,
        other => {
            return Err(SceneLoadError::Invalid(format!(
                "{label} draws in mode {other:?}, and Scene.load draws only triangles"
            )));
        }
    };
    let order = match indices {
        Some(indices) => indices,
        None => {
            let Ok(count) = u32::try_from(vertex_count) else {
                return Err(SceneLoadError::Invalid(format!(
                    "{label} has {vertex_count} vertices, more than 32-bit indices reach"
                )));
            };
            let mut order = reserve(vertex_count, label)?;
            order.extend(0..count);
            order
        }
    };
    Ok(Some(expand(label, &order)?))
}

/// Reserve the list a strip or fan of `steps` triangles expands into.
fn reserve_list(label: &str, steps: usize) -> Result<Vec<u32>, SceneLoadError> {
    let Some(count) = steps.checked_mul(3) else {
        return Err(SceneLoadError::Invalid(format!(
            "{label} expands into more triangle indices than this platform addresses"
        )));
    };
    reserve(count, label)
}

/// One triangle per step of a strip, every second one flipped back to the
/// strip's winding.
fn expand_strip(label: &str, order: &[u32]) -> Result<Vec<u32>, SceneLoadError> {
    let mut list = reserve_list(label, order.len().saturating_sub(2))?;
    list.extend(
        order
            .windows(3)
            .enumerate()
            .flat_map(|(step, corners)| match step % 2 {
                0 => [corners[0], corners[1], corners[2]],
                _ => [corners[1], corners[0], corners[2]],
            }),
    );
    Ok(list)
}

/// One triangle per step of a fan, each sharing the first vertex.
fn expand_fan(label: &str, order: &[u32]) -> Result<Vec<u32>, SceneLoadError> {
    let Some((&hub, rim)) = order.split_first() else {
        return Ok(Vec::new());
    };
    let mut list = reserve_list(label, rim.len().saturating_sub(1))?;
    list.extend(rim.windows(2).flat_map(|edge| [hub, edge[0], edge[1]]));
    Ok(list)
}

/// Refuse a triangle list that ends partway through a triangle, as glTF requires
/// of TRIANGLES, and an index that points past the last vertex. Strips and fans
/// arrive here expanded into lists, so they always hold whole triangles.
fn check_indices(
    label: &str,
    indices: Option<&[u32]>,
    vertex_count: usize,
) -> Result<(), SceneLoadError> {
    let (count, what) = match indices {
        Some(indices) => (indices.len(), "indices"),
        None => (vertex_count, "vertices and no indices"),
    };
    if !count.is_multiple_of(3) {
        return Err(SceneLoadError::Invalid(format!(
            "{label} has {count} {what}, and a triangle list takes a multiple of 3"
        )));
    }
    let past_end = indices
        .unwrap_or_default()
        .iter()
        .find(|&&index| index as usize >= vertex_count);
    match past_end {
        Some(index) => Err(SceneLoadError::Invalid(format!(
            "{label} index {index} points past the last of {vertex_count} vertices"
        ))),
        None => Ok(()),
    }
}

/// Smooth per-vertex normals from positions + an optional index buffer.
/// Accumulates the un-normalized cross product of each triangle into the
/// vertex slots it touches (area-weighted contribution by construction);
/// the final normalize step turns the accumulated direction into a unit
/// vector. Degenerate vertices (zero accumulated direction, or untouched
/// by any triangle) fall back to a forward-Z normal so the BRDF stays
/// finite — matches the glTF 2.0 default for POSITION-only meshes.
fn compute_vertex_normals(
    label: &str,
    positions: &[[f32; 3]],
    indices: Option<&[u32]>,
) -> Result<Vec<[f32; 3]>, SceneLoadError> {
    let n_verts = positions.len();
    let mut accum = reserve(n_verts, label)?;
    accum.resize(n_verts, Vec3::ZERO);
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
        for tri in idx.as_chunks::<3>().0 {
            visit_triangle(
                tri[0] as usize,
                tri[1] as usize,
                tri[2] as usize,
                &mut accum,
            );
        }
    } else {
        let mut i = 0;
        while i + 2 < n_verts {
            visit_triangle(i, i + 1, i + 2, &mut accum);
            i += 3;
        }
    }
    let mut normals = reserve(n_verts, label)?;
    normals.extend(accum.iter().map(|v| {
        v.try_normalize()
            .unwrap_or(Vec3::new(0.0, 0.0, 1.0))
            .to_array()
    }));
    Ok(normals)
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

/// Build the PBR Material of a glTF material. Each texture slot becomes a map
/// of its image's decoded pixels, which `pixels` keeps for every slot that reads them.
fn build_material(
    gltf_material: &gltf::Material<'_>,
    images: &[gltf::image::Data],
    pixels: &mut HashMap<usize, Arc<SourceData>>,
) -> Result<Material, SceneLoadError> {
    let pbr = gltf_material.pbr_metallic_roughness();
    let mut material = Material::pbr()
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

    if let Some(scale) = gltf_material.normal_texture().map(|info| info.scale()) {
        material = material.normal_scale(scale);
    }
    if let Some(strength) = gltf_material
        .occlusion_texture()
        .map(|info| info.strength())
    {
        material = material.occlusion_strength(strength);
    }

    let mut read = |texture: gltf::Texture<'_>| convert_image(&texture, images, pixels);
    if let Some(info) = pbr.base_color_texture() {
        material.set_map(TextureSlot::BaseColor, read(info.texture())?);
        apply_uv(&material, TextureSlot::BaseColor, read_info_uv(&info))?;
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        material.set_map(TextureSlot::MetallicRoughness, read(info.texture())?);
        apply_uv(
            &material,
            TextureSlot::MetallicRoughness,
            read_info_uv(&info),
        )?;
    }
    if let Some(info) = gltf_material.normal_texture() {
        material.set_map(TextureSlot::Normal, read(info.texture())?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Normal, uv)?;
    }
    if let Some(info) = gltf_material.occlusion_texture() {
        material.set_map(TextureSlot::Occlusion, read(info.texture())?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Occlusion, uv)?;
    }
    if let Some(info) = gltf_material.emissive_texture() {
        material.set_map(TextureSlot::Emissive, read(info.texture())?);
        apply_uv(&material, TextureSlot::Emissive, read_info_uv(&info))?;
    }

    Ok(material)
}

/// The extension that gives a glTF texture reference its own UV transform.
const TEXTURE_TRANSFORM: &str = "KHR_texture_transform";

/// The UVs a glTF texture reference reads: the `TEXCOORD_<n>` set it
/// names, and the transform from its `KHR_texture_transform`.
struct TextureUv {
    tex_coord: u32,
    transform: UvTransform,
}

/// Read the UVs of a texture reference the gltf crate types, whose
/// `KHR_texture_transform` it parses. The extension's `texCoord` wins
/// over the reference's own.
fn read_info_uv(info: &gltf::texture::Info<'_>) -> TextureUv {
    match info.texture_transform() {
        Some(transform) => TextureUv {
            tex_coord: transform.tex_coord().unwrap_or(info.tex_coord()),
            transform: UvTransform {
                offset: transform.offset(),
                scale: transform.scale(),
                rotation: transform.rotation(),
            },
        },
        None => TextureUv {
            tex_coord: info.tex_coord(),
            transform: UvTransform::default(),
        },
    }
}

/// Read the UVs of a normal or occlusion texture reference. The gltf crate
/// keeps their `KHR_texture_transform` as raw JSON, parsed here as it parses
/// the typed slots, so a `null` extension counts as no extension.
fn read_extension_uv(
    tex_coord: u32,
    extension: Option<&gltf::json::Value>,
) -> Result<TextureUv, SceneLoadError> {
    use gltf::json::extensions::texture::TextureTransform;
    let parsed: Option<TextureTransform> = match extension {
        Some(value) => gltf::json::deserialize::from_value(value.clone())
            .map_err(|error| SceneLoadError::Invalid(format!("{TEXTURE_TRANSFORM}: {error}")))?,
        None => None,
    };
    let Some(transform) = parsed else {
        return Ok(TextureUv {
            tex_coord,
            transform: UvTransform::default(),
        });
    };
    Ok(TextureUv {
        tex_coord: transform.tex_coord.unwrap_or(tex_coord),
        transform: UvTransform {
            offset: transform.offset.0,
            scale: transform.scale.0,
            rotation: transform.rotation.0,
        },
    })
}

/// Point one material slot at the UV set and transform its glTF texture reads.
/// The PBR shader samples two UV sets, so `TEXCOORD_2` and above are an error.
fn apply_uv(material: &Material, slot: TextureSlot, uv: TextureUv) -> Result<(), SceneLoadError> {
    let Ok(set) = UvSet::try_from(uv.tex_coord) else {
        return Err(SceneLoadError::Invalid(format!(
            "glTF {} reads TEXCOORD_{}, and the PBR material samples TEXCOORD_0 and TEXCOORD_1 only",
            name_slot(slot),
            uv.tex_coord
        )));
    };
    material.set_slot_uv(slot, set, uv.transform);
    Ok(())
}

/// The glTF property that holds a slot's texture reference.
fn name_slot(slot: TextureSlot) -> &'static str {
    match slot {
        TextureSlot::BaseColor => "baseColorTexture",
        TextureSlot::MetallicRoughness => "metallicRoughnessTexture",
        TextureSlot::Normal => "normalTexture",
        TextureSlot::Occlusion => "occlusionTexture",
        TextureSlot::Emissive => "emissiveTexture",
    }
}

/// Convert a glTF texture reference into the source a Material map keeps, with the
/// texture's own sampler. Each image decodes once into RGBA8, which `pixels` shares
/// with every later reference to it. An image with fewer bytes than its size needs is an error.
fn convert_image(
    texture: &gltf::Texture<'_>,
    images: &[gltf::image::Data],
    pixels: &mut HashMap<usize, Arc<SourceData>>,
) -> Result<TextureSource, SceneLoadError> {
    let index = texture.source().index();
    let image = images
        .get(index)
        .ok_or_else(|| SceneLoadError::Invalid("glTF texture references missing image".into()))?;
    let data = match pixels.entry(index) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let rgba = decode_image(image)?;
            entry
                .insert(Arc::new(SourceData::Bytes(rgba.into_raw())))
                .clone()
        }
    };
    let options = TextureOptions {
        size: Some(crate::Size::from([image.width, image.height])),
        sampler: map_sampler_options(&texture.sampler()),
        ..Default::default()
    };
    Ok(TextureSource::new(data, options))
}

/// Decode imported glTF pixels into RGBA8.
/// The importer stores 16-bit and float pixels in native byte order,
/// so those formats read their channels back with `from_ne_bytes`.
fn decode_image(image: &gltf::image::Data) -> Result<image::RgbaImage, SceneLoadError> {
    use gltf::image::Format;
    use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
    let (width, height) = (image.width, image.height);
    let bytes = || image.pixels.clone();
    let wide = || -> Vec<u16> {
        let (pairs, _) = image.pixels.as_chunks::<2>();
        pairs.iter().map(|pair| u16::from_ne_bytes(*pair)).collect()
    };
    let float = || -> Vec<f32> {
        let (quads, _) = image.pixels.as_chunks::<4>();
        quads.iter().map(|quad| f32::from_ne_bytes(*quad)).collect()
    };
    let decoded = match image.format {
        Format::R8 => ImageBuffer::<Luma<u8>, _>::from_raw(width, height, bytes())
            .map(DynamicImage::ImageLuma8),
        Format::R8G8 => ImageBuffer::<LumaA<u8>, _>::from_raw(width, height, bytes())
            .map(DynamicImage::ImageLumaA8),
        Format::R8G8B8 => {
            ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, bytes()).map(DynamicImage::ImageRgb8)
        }
        Format::R8G8B8A8 => ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, bytes())
            .map(DynamicImage::ImageRgba8),
        Format::R16 => ImageBuffer::<Luma<u16>, _>::from_raw(width, height, wide())
            .map(DynamicImage::ImageLuma16),
        Format::R16G16 => ImageBuffer::<LumaA<u16>, _>::from_raw(width, height, wide())
            .map(DynamicImage::ImageLumaA16),
        Format::R16G16B16 => ImageBuffer::<Rgb<u16>, _>::from_raw(width, height, wide())
            .map(DynamicImage::ImageRgb16),
        Format::R16G16B16A16 => ImageBuffer::<Rgba<u16>, _>::from_raw(width, height, wide())
            .map(DynamicImage::ImageRgba16),
        Format::R32G32B32FLOAT => ImageBuffer::<Rgb<f32>, _>::from_raw(width, height, float())
            .map(DynamicImage::ImageRgb32F),
        Format::R32G32B32A32FLOAT => ImageBuffer::<Rgba<f32>, _>::from_raw(width, height, float())
            .map(DynamicImage::ImageRgba32F),
    };
    decoded.map(|image| image.to_rgba8()).ok_or_else(|| {
        SceneLoadError::Invalid(format!(
            "glTF {:?} image has fewer bytes than {width} × {height} pixels need",
            image.format
        ))
    })
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
        gltf::camera::Projection::Perspective(p) => (p.znear(), p.zfar()),
        gltf::camera::Projection::Orthographic(o) => (o.znear(), Some(o.zfar())),
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
        gltf::camera::Projection::Perspective(p) => {
            // glTF's aspectRatio is optional; fall back to 1.0 (a common
            // default that gets overridden when the user retargets to a
            // specific render target).
            let aspect = p.aspect_ratio().unwrap_or(1.0);
            match zfar {
                Some(far) => Camera::perspective(p.yfov(), aspect, near, far),
                None => Camera::create_infinite_perspective(p.yfov(), aspect, near),
            }
            .map_err(blame_camera)?
        }
        gltf::camera::Projection::Orthographic(o) => {
            let half_w = o.xmag();
            let half_h = o.ymag();
            Camera::orthographic(-half_w, half_w, -half_h, half_h, near, o.zfar())
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
    let target_v = translation + forward;
    camera
        .look_at(eye, target_v.into(), up.into())
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
mod tests {
    use super::*;
    use crate::Renderable;
    use crate::mesh::VertexData;

    /// The minimal triangle: positions only, read through accessor 0 from
    /// bufferView 0. Views 1 (12 bytes) and 2 (24 bytes) sit unused, so a
    /// test can point a new accessor at them with a plain text replace.
    const TRIANGLE_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":72}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":12,"byteOffset":36},{"buffer":0,"byteLength":24,"byteOffset":48}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;

    /// Build a minimal valid .glb in memory: one triangle, positions only.
    fn build_minimal_triangle_glb() -> Vec<u8> {
        build_triangle_glb(TRIANGLE_JSON)
    }

    /// Pack `json` with the triangle's BIN chunk: three vec3 positions, the
    /// u32 indices `[0, 1, 2]` and 24 bytes of `[0.5, 0.5]` float pairs.
    fn build_triangle_glb(json: &str) -> Vec<u8> {
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        bin.extend([0u32, 1, 2].iter().flat_map(|i| i.to_le_bytes()));
        bin.extend([0.5f32; 6].iter().flat_map(|f| f.to_le_bytes()));
        build_glb(json, &bin)
    }

    /// Apply text patches to the minimal triangle's JSON and pack it.
    fn mutate_triangle(patches: &[(&str, &str)]) -> Vec<u8> {
        let json = patches
            .iter()
            .fold(TRIANGLE_JSON.to_string(), |json, (from, to)| {
                assert!(json.contains(from), "patch target {from:?} not in the JSON");
                json.replace(from, to)
            });
        build_triangle_glb(&json)
    }

    /// Wrap a JSON chunk and a BIN chunk in a .glb container.
    fn build_glb(json: &str, bin: &[u8]) -> Vec<u8> {
        let bin_len = bin.len() as u32;
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
        glb.extend_from_slice(bin);
        glb
    }

    /// Same triangle as `build_minimal_triangle_glb` but with one extra
    /// node that holds a perspective camera, and one directional light
    /// declared via `KHR_lights_punctual`. Gives the filter tests a real
    /// camera + light to drop. The scene has three nodes: mesh (0),
    /// camera (1) and light (2).
    const CAMERA_LIGHT_JSON: &str = r#"{"asset":{"version":"2.0"},"extensionsUsed":["KHR_lights_punctual"],"extensions":{"KHR_lights_punctual":{"lights":[{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}]}},"scene":0,"scenes":[{"nodes":[0,1,2]}],"nodes":[{"mesh":0},{"camera":0},{"extensions":{"KHR_lights_punctual":{"light":0}}}],"cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}]}"#;

    fn build_triangle_with_camera_and_light_glb() -> Vec<u8> {
        build_camera_light_glb(CAMERA_LIGHT_JSON)
    }

    /// Pack a variant of `CAMERA_LIGHT_JSON` with the triangle's positions.
    fn build_camera_light_glb(json: &str) -> Vec<u8> {
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        build_glb(json, &bin)
    }

    const CAMERAS: &str = r#""cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}}]"#;

    const DIRECTIONAL_LIGHT: &str =
        r#"{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}"#;

    #[test]
    fn load_names_the_light_at_fault() {
        // Light 0 is fine; the node points at light 1, whose range is negative.
        let lights = format!(r#"{DIRECTIONAL_LIGHT},{{"type":"point","range":-5.0}}"#);
        let json = CAMERA_LIGHT_JSON
            .replace(DIRECTIONAL_LIGHT, &lights)
            .replace(
                r#"{"KHR_lights_punctual":{"light":0}}"#,
                r#"{"KHR_lights_punctual":{"light":1}}"#,
            );
        assert_invalid(
            build_camera_light_glb(&json),
            "glTF light 1: Light::set_range",
        );
    }

    #[test]
    fn load_refuses_a_negative_light_range() {
        let json = CAMERA_LIGHT_JSON.replace(DIRECTIONAL_LIGHT, r#"{"type":"point","range":-5.0}"#);
        assert_invalid(
            build_camera_light_glb(&json),
            "glTF light 0: Light::set_range expects a non-negative number, got -5",
        );
    }

    /// Load a variant of the camera file and expect `SceneLoadError::Camera`
    /// with a message that holds `needle`.
    fn assert_camera_error(json: &str, needle: &str) {
        match Scene::load(build_camera_light_glb(json)) {
            Err(error @ SceneLoadError::Camera { .. }) => {
                let message = error.to_string();
                assert!(message.contains(needle), "{message:?} lacks {needle:?}")
            }
            Err(other) => panic!("expected SceneLoadError::Camera, got {other:?}"),
            Ok(_) => panic!("expected SceneLoadError::Camera, got a Scene"),
        }
    }

    #[test]
    fn load_refuses_a_camera_with_no_field_of_view() {
        assert_camera_error(
            &CAMERA_LIGHT_JSON.replace(r#""yfov":1.0"#, r#""yfov":0.0"#),
            "glTF camera 0: Camera::perspective",
        );
    }

    #[test]
    fn load_refuses_a_camera_node_with_zero_scale() {
        let json =
            CAMERA_LIGHT_JSON.replace(r#"{"camera":0}"#, r#"{"camera":0,"scale":[0.0,0.0,0.0]}"#);
        assert_camera_error(&json, "glTF camera 0: Camera::look_at");
    }

    #[test]
    fn load_names_the_camera_at_fault() {
        // Camera 0 is fine; the node points at camera 1, whose yfov is 0 and which
        // has no zfar, so the error names its lens and no far plane the file never wrote.
        let cameras = r#""cameras":[{"type":"perspective","perspective":{"aspectRatio":1.5,"yfov":1.0,"znear":0.1,"zfar":100.0}},{"type":"perspective","perspective":{"yfov":0.0,"znear":0.1}}]"#;
        let json = CAMERA_LIGHT_JSON
            .replace(CAMERAS, cameras)
            .replace(r#"{"camera":0}"#, r#"{"camera":1}"#);
        assert_camera_error(
            &json,
            "glTF camera 1: a perspective camera with no zfar needs finite values with 0 < yfov < π, a nonzero aspect ratio and a positive znear, got yfov 0, aspect ratio 1, znear 0.1",
        );
    }

    #[test]
    fn load_takes_a_camera_with_no_far_plane() {
        // glTF reads an absent zfar as an infinite projection: nothing ahead of
        // the near plane clips, however far, and any near plane builds one. Far
        // enough out f32 rounds the depth to 1, which the LessEqual test draws.
        let depth_at = |znear: &str, distance: f32| {
            let json = CAMERA_LIGHT_JSON.replace(
                r#""znear":0.1,"zfar":100.0"#,
                &format!(r#""znear":{znear}"#),
            );
            let scene = Scene::load(build_camera_light_glb(&json)).expect("load");
            let cameras = scene.cameras();
            let view_proj = glam::Mat4::from_cols_array_2d(&cameras[0].view_proj());
            assert!(view_proj.is_finite(), "{view_proj:?}");
            let clip = view_proj * glam::Vec4::new(0.0, 0.0, -distance, 1.0);
            clip.z / clip.w
        };
        for (znear, distance) in [("0.001", 1.0e6), ("0.1", 1.0e9), ("1.0e16", 1.0e30)] {
            let depth = depth_at(znear, distance);
            assert!(
                (0.0..=1.0).contains(&depth),
                "znear {znear}: a point {distance} ahead lands at depth {depth}"
            );
        }
    }

    #[test]
    fn load_refuses_a_camera_whose_far_plane_is_nearer() {
        let json =
            CAMERA_LIGHT_JSON.replace(r#""znear":0.1,"zfar":100.0"#, r#""znear":10.0,"zfar":1.0"#);
        assert_invalid(
            build_camera_light_glb(&json),
            "camera 0 has zfar 1 at or before znear 10",
        );
    }

    #[test]
    fn load_default_options_keeps_gltf_camera_and_light() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(bytes).expect("load");
        assert_eq!(scene.models().len(), 1, "geometry stays");
        assert_eq!(
            scene.cameras().len(),
            1,
            "default options load the embedded camera"
        );
        assert_eq!(
            scene.lights().len(),
            1,
            "default options load the embedded light"
        );
    }

    #[test]
    fn cameras_filter_skips_gltf_camera_nodes() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(SceneSource::gltf(bytes).cameras(false)).expect("load");
        assert_eq!(scene.models().len(), 1, "geometry stays");
        assert!(
            scene.cameras().is_empty(),
            "cameras filter drops the camera node"
        );
        assert_eq!(scene.lights().len(), 1, "lights still load by default");
    }

    #[test]
    fn lights_filter_skips_khr_lights_punctual() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene = Scene::load(SceneSource::gltf(bytes).lights(false)).expect("load");
        assert_eq!(scene.models().len(), 1);
        assert_eq!(scene.cameras().len(), 1, "cameras still load by default");
        assert!(
            scene.lights().is_empty(),
            "lights filter drops the light node"
        );
    }

    #[test]
    fn both_filters_off_yields_geometry_only_scene() {
        let bytes = build_triangle_with_camera_and_light_glb();
        let scene =
            Scene::load(SceneSource::gltf(bytes).cameras(false).lights(false)).expect("load");
        assert_eq!(scene.models().len(), 1, "geometry survives every filter");
        assert!(scene.cameras().is_empty());
        assert!(scene.lights().is_empty());
    }

    #[test]
    fn load_minimal_triangle_glb_returns_scene_with_one_model() {
        let bytes = build_minimal_triangle_glb();
        let scene = Scene::load(bytes).expect("load triangle.glb");
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
        // Exercises the From<&str> -> SceneSource conversion (bare path, no
        // SceneSource named in the call); the actual file IO error is what
        // we want to see.
        let result = Scene::load("/definitely/not/a/real/path.glb");
        assert!(result.is_err(), "expected a load error for a bogus path");
    }

    /// Load `bytes` and expect `SceneLoadError::Invalid` naming `needle`.
    fn assert_invalid(bytes: Vec<u8>, needle: &str) {
        match Scene::load(bytes) {
            Err(SceneLoadError::Invalid(message)) => {
                assert!(message.contains(needle), "{message:?} lacks {needle:?}")
            }
            Err(other) => panic!("expected SceneLoadError::Invalid, got {other:?}"),
            Ok(_) => panic!("expected SceneLoadError::Invalid, got a Scene"),
        }
    }

    const POSITION_VIEW: &str = r#"{"buffer":0,"byteLength":36,"byteOffset":0}"#;
    const LAST_ACCESSOR: &str = r#""max":[0.5,0.5,0.0]}]"#;
    const ATTRIBUTES: &str = r#"{"POSITION":0}"#;
    const MODE: &str = r#""mode":4}"#;

    #[test]
    fn load_refuses_a_stride_shorter_than_the_element() {
        let bytes = mutate_triangle(&[(
            POSITION_VIEW,
            r#"{"buffer":0,"byteLength":36,"byteOffset":0,"byteStride":4}"#,
        )]);
        assert_invalid(bytes, "byteStride 4");
    }

    #[test]
    fn load_refuses_float_indices() {
        let bytes = mutate_triangle(&[
            (MODE, r#""indices":1,"mode":4}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":1,"componentType":5126,"count":3,"type":"SCALAR"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(indices) holds F32 Scalar");
    }

    #[test]
    fn load_refuses_unsigned_int_tex_coords() {
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":3,"type":"VEC2"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(TEXCOORD_0) holds U32 Vec2");
    }

    #[test]
    fn load_refuses_vec2_colors() {
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, r#"{"POSITION":0,"COLOR_0":1}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(COLOR_0) holds F32 Vec2");
    }

    #[test]
    fn load_refuses_an_empty_position_accessor() {
        let bytes =
            mutate_triangle(&[(r#""count":3,"type":"VEC3""#, r#""count":0,"type":"VEC3""#)]);
        assert_invalid(bytes, "(POSITION) has count 0");
    }

    #[test]
    fn load_refuses_an_empty_index_accessor() {
        let bytes = mutate_triangle(&[
            (MODE, r#""indices":1,"mode":4}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":1,"componentType":5125,"count":0,"type":"SCALAR"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(indices) has count 0");
    }

    #[test]
    fn load_refuses_a_count_that_overflows_the_byte_range() {
        let huge = mutate_triangle(&[(
            r#""count":3,"type":"VEC3""#,
            r#""count":4611686018427387904,"type":"VEC3""#,
        )]);
        assert_invalid(huge, "holds 4611686018427387904 elements");
        let largest = mutate_triangle(&[(
            r#""count":3,"type":"VEC3""#,
            r#""count":4294967295,"type":"VEC3""#,
        )]);
        assert_invalid(largest, "reads past the end of bufferView 0");
    }

    #[test]
    fn load_refuses_an_accessor_past_its_view() {
        // A declared NORMAL the reader can't reach is an error, not a cue to
        // compute normals: its last element would end at byte 28 of 24.
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, r#"{"POSITION":0,"NORMAL":1}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"byteOffset":4,"componentType":5126,"count":2,"type":"VEC3"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(NORMAL) reads past the end of bufferView 2");
    }

    #[test]
    fn load_refuses_a_view_past_its_buffer() {
        let bytes = mutate_triangle(&[
            (
                r#"{"buffer":0,"byteLength":24,"byteOffset":48}"#,
                r#"{"buffer":0,"byteLength":24,"byteOffset":64}"#,
            ),
            (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"}]"#,
            ),
        ]);
        assert_invalid(bytes, "runs past the end of buffer 0");
    }

    #[test]
    fn load_refuses_a_node_that_is_its_own_child() {
        let bytes = mutate_triangle(&[(
            r#""nodes":[{"mesh":0}]"#,
            r#""nodes":[{"mesh":0,"children":[0]}]"#,
        )]);
        assert_invalid(bytes, "glTF node 0 is reached twice");
    }

    #[test]
    fn load_refuses_a_child_with_two_parents() {
        let bytes = mutate_triangle(&[(
            r#""scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}]"#,
            r#""scenes":[{"nodes":[0,1]}],"nodes":[{"children":[2]},{"children":[2]},{"mesh":0}]"#,
        )]);
        assert_invalid(bytes, "glTF node 2 is reached twice");
    }

    #[test]
    fn load_names_the_mesh_and_primitive_at_fault() {
        // Mesh 0 is unused; the node draws mesh 1, whose third primitive is points.
        let triangle = r#"{"attributes":{"POSITION":0},"mode":4}"#;
        let meshes = format!(
            r#""meshes":[{{"primitives":[{triangle}]}},{{"primitives":[{triangle},{triangle},{{"attributes":{{"POSITION":0}},"mode":0}}]}}]"#
        );
        let bytes = mutate_triangle(&[
            (
                r#""meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}]"#,
                &meshes,
            ),
            (r#""nodes":[{"mesh":0}]"#, r#""nodes":[{"mesh":1}]"#),
        ]);
        assert_invalid(bytes, "glTF mesh 1 primitive 2 draws in mode Points");
    }

    #[test]
    fn load_refuses_a_cycle_between_two_nodes() {
        let bytes = mutate_triangle(&[(
            r#""nodes":[{"mesh":0}]"#,
            r#""nodes":[{"mesh":0,"children":[1]},{"children":[0]}]"#,
        )]);
        assert_invalid(bytes, "glTF node 0 is reached twice");
    }

    const POSITION_ACCESSOR: &str = r#"{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}"#;

    /// The triangle with POSITION as a sparse accessor of `count` elements
    /// and no bufferView: zeros, except element 0 read from view 0.
    fn sparse_positions(count: &str) -> Vec<u8> {
        let sparse = r#"{"componentType":5126,"count":COUNT,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0],"sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":0}}}"#;
        mutate_triangle(&[(POSITION_ACCESSOR, &sparse.replace("COUNT", count))])
    }

    #[test]
    fn load_reads_a_sparse_accessor_without_a_view() {
        let scene = Scene::load(sparse_positions("3")).expect("load sparse POSITION");
        assert_eq!(scene.models().len(), 1);
    }

    #[test]
    fn load_refuses_a_sparse_accessor_too_large_to_read() {
        // Without a view, nothing in the file bounds the count, and
        // collecting 2^62 positions overflowed the allocation.
        assert_invalid(
            sparse_positions("4611686018427387904"),
            "(POSITION) holds 4611686018427387904 elements",
        );
        assert_invalid(
            sparse_positions("4294967296"),
            "(POSITION) holds 4294967296 elements",
        );
    }

    /// Pack a .glb whose BIN chunk holds the triangle's positions, one u32
    /// sparse index of 0, one vec3 sparse value, then `extra` bytes.
    fn build_sparse_glb(json: &str, extra: &[u8]) -> Vec<u8> {
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        bin.extend(0u32.to_le_bytes());
        bin.extend([0.0f32, 0.1, 0.0].iter().flat_map(|f| f.to_le_bytes()));
        bin.extend_from_slice(extra);
        build_glb(json, &bin)
    }

    #[test]
    fn load_ignores_a_bufferless_animation_track() {
        // A 100-key translation track whose output is sparse without a view,
        // the shape glTF-Transform's sparse() writes. The loader never reads
        // animations, so the track's 1200 bytes of zeros don't count.
        let times: Vec<u8> = (0..100)
            .flat_map(|key| (key as f32 / 3.0).to_le_bytes())
            .collect();
        let json = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":452}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":36},{"buffer":0,"byteLength":12,"byteOffset":40},{"buffer":0,"byteLength":400,"byteOffset":52}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]},{"bufferView":3,"componentType":5126,"count":100,"type":"SCALAR","min":[0.0],"max":[33.0]},{"componentType":5126,"count":100,"type":"VEC3","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}],"animations":[{"channels":[{"sampler":0,"target":{"node":0,"path":"translation"}}],"samplers":[{"input":1,"output":2}]}]}"#;
        let scene = Scene::load(build_sparse_glb(json, &times)).expect("load");
        assert_eq!(scene.models().len(), 1);
    }

    const UNUSED_ACCESSOR_JSON: &str = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":52}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":36},{"buffer":0,"byteLength":12,"byteOffset":40}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]},UNUSED]}"#;

    #[test]
    fn load_ignores_an_unused_bufferless_accessor() {
        // A sparse accessor 1000 elements long that nothing references.
        let unused = r#"{"componentType":5126,"count":1000,"type":"VEC3","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}"#;
        let json = UNUSED_ACCESSOR_JSON.replace("UNUSED", unused);
        let scene = Scene::load(build_sparse_glb(&json, &[])).expect("load");
        assert_eq!(scene.models().len(), 1);
    }

    #[test]
    fn load_reports_an_all_zero_accessor_as_a_parse_error() {
        // glTF reads an accessor with neither a view nor sparse values as all
        // zeros, but the glTF parser refuses it before the loader runs.
        let json = UNUSED_ACCESSOR_JSON.replace(
            "UNUSED",
            r#"{"componentType":5126,"count":1000,"type":"VEC3"}"#,
        );
        assert!(matches!(
            Scene::load(build_sparse_glb(&json, &[])),
            Err(SceneLoadError::Gltf(_))
        ));
    }

    #[test]
    fn load_reads_a_bufferless_color_beside_dense_positions() {
        // 99 positions take 1188 bytes. Their sparse COLOR_0 spans 1584 bytes
        // of zeros, more than the file holds, but it can't outgrow POSITION.
        let json = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0,"COLOR_0":1},"mode":4}]}],"buffers":[{"byteLength":1208}],"bufferViews":[{"buffer":0,"byteLength":1188,"byteOffset":0},{"buffer":0,"byteLength":4,"byteOffset":1188},{"buffer":0,"byteLength":16,"byteOffset":1192}],"accessors":[{"bufferView":0,"componentType":5126,"count":99,"type":"VEC3","min":[0.0,0.0,0.0],"max":[33.0,1.0,0.0]},{"componentType":5126,"count":99,"type":"VEC4","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":2}}}]}"#;
        let mut bin: Vec<u8> = (0..99)
            .flat_map(|vertex| [vertex as f32 / 3.0, (vertex % 3 == 1) as u8 as f32, 0.0])
            .flat_map(|value| value.to_le_bytes())
            .collect();
        bin.extend(0u32.to_le_bytes());
        bin.extend([1.0f32; 4].iter().flat_map(|f| f.to_le_bytes()));
        let scene = Scene::load(build_glb(json, &bin)).expect("load");
        assert_eq!(scene.models().len(), 1);
    }

    #[test]
    fn load_refuses_a_sparse_accessor_larger_than_the_file() {
        // The file holds 72 bytes of buffers. Its one sparse value can't stand
        // for 100,000 positions, and a count just under u32::MAX would ask for
        // 51 GB that an overcommitting system grants and then can't fill.
        assert_invalid(
            sparse_positions("100000"),
            "glTF accessor 0 (POSITION) spans 1200000 bytes without a bufferView, more than the 72 bytes",
        );
        assert_invalid(
            sparse_positions("4294967295"),
            "glTF accessor 0 (POSITION) spans 51539607540 bytes without a bufferView",
        );
    }

    #[test]
    fn reserve_refuses_a_count_the_allocator_cant_give() {
        // Every Vec the loader sizes from a count goes through `reserve`, so a
        // refused allocation is an error there instead of an abort.
        let refused = reserve::<[f32; 3]>(usize::MAX, "glTF accessor 0 (POSITION)");
        let message = refused
            .expect_err("usize::MAX positions overflow")
            .to_string();
        assert!(
            message.contains("glTF accessor 0 (POSITION) needs"),
            "{message}"
        );
    }

    #[test]
    fn load_refuses_sparse_indices_too_large_to_read() {
        let indices = r#""max":[0.5,0.5,0.0]},{"componentType":5125,"count":4611686018427387904,"type":"SCALAR","sparse":{"count":1,"indices":{"bufferView":1,"componentType":5125},"values":{"bufferView":1}}}]"#;
        let bytes =
            mutate_triangle(&[(MODE, r#""indices":1,"mode":4}"#), (LAST_ACCESSOR, indices)]);
        assert_invalid(bytes, "(indices) holds 4611686018427387904 elements");
    }

    #[test]
    fn load_reads_valid_indices_and_uvs() {
        // The checks must not refuse the accessors they exist to protect.
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, r#"{"POSITION":0,"TEXCOORD_0":1}"#),
            (MODE, r#""indices":2,"mode":4}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"},{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR"}]"#,
            ),
        ]);
        let scene = Scene::load(bytes).expect("load");
        assert_eq!(scene.models().len(), 1);
    }

    /// A unit quad, four positions read through accessor 0, drawn as a strip.
    const QUAD_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":5}]}],"buffers":[{"byteLength":48}],"bufferViews":[{"buffer":0,"byteLength":48,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":4,"type":"VEC3","min":[0.0,0.0,0.0],"max":[1.0,1.0,0.0]}],"asset":{"version":"2.0"}}"#;

    /// Load the quad with `mode` and return the indices its mesh draws.
    fn load_quad_indices(mode: &str) -> Vec<u32> {
        #[rustfmt::skip]
        let positions: [f32; 12] = [
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            1.0, 1.0, 0.0,
        ];
        let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let json = QUAD_JSON.replace(r#""mode":5"#, mode);
        let scene = Scene::load(build_glb(&json, &bin)).expect("load quad");
        let models = scene.models();
        let mesh = &models[0].mesh().object;
        assert!(*mesh.indices_overridden.read(), "{mode} should set indices");
        mesh.indices.read().clone()
    }

    #[test]
    fn load_expands_a_triangle_strip() {
        // Every second strip triangle swaps its first two corners, so both
        // triangles keep the strip's winding.
        assert_eq!(load_quad_indices(r#""mode":5"#), [0, 1, 2, 2, 1, 3]);
    }

    #[test]
    fn load_expands_a_triangle_fan() {
        assert_eq!(load_quad_indices(r#""mode":6"#), [0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn load_refuses_points_and_lines() {
        for (mode, name) in [
            ("0", "Points"),
            ("1", "Lines"),
            ("2", "LineLoop"),
            ("3", "LineStrip"),
        ] {
            let bytes = mutate_triangle(&[(MODE, &format!(r#""mode":{mode}}}"#))]);
            assert_invalid(
                bytes,
                &format!("glTF mesh 0 primitive 0 draws in mode {name}"),
            );
        }
    }

    #[test]
    fn load_refuses_an_attribute_shorter_than_positions() {
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, r#"{"POSITION":0,"NORMAL":1}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":2,"type":"VEC3"}]"#,
            ),
        ]);
        assert_invalid(bytes, "(NORMAL) holds 2 elements for 3 positions");
    }

    // Story: glTF draws TRIANGLES in threes, so a list that ends partway through a
    // triangle, indexed or not, is refused with the primitive named.
    #[test]
    fn load_refuses_a_partial_triangle_list() {
        #[rustfmt::skip]
        let positions: [f32; 12] = [
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            1.0, 1.0, 0.0,
        ];
        let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let json = QUAD_JSON.replace(r#""mode":5"#, r#""mode":4"#);
        assert_invalid(
            build_glb(&json, &bin),
            "glTF mesh 0 primitive 0 has 4 vertices and no indices, and a triangle list takes a multiple of 3",
        );

        let bytes = mutate_triangle(&[
            (MODE, r#""indices":1,"mode":4}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":4,"type":"SCALAR"}]"#,
            ),
        ]);
        assert_invalid(
            bytes,
            "glTF mesh 0 primitive 0 has 4 indices, and a triangle list takes a multiple of 3",
        );
    }

    #[test]
    fn load_refuses_an_index_past_the_last_vertex() {
        // View 2 holds 0.5 floats, which read as u32 0x3F00_0000.
        let bytes = mutate_triangle(&[
            (MODE, r#""indices":1,"mode":4}"#),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5125,"count":3,"type":"SCALAR"}]"#,
            ),
        ]);
        assert_invalid(
            bytes,
            "glTF mesh 0 primitive 0 index 1056964608 points past the last of 3 vertices",
        );
    }

    /// The minimal triangle with a base color texture read from an embedded PNG.
    const TEXTURED_JSON: &str = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"material":0,"mode":4}]}],"materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}],"textures":[{"source":0}],"images":[{"bufferView":1,"mimeType":"image/png"}],"buffers":[{"byteLength":BUFFER_LENGTH}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0},{"buffer":0,"byteLength":PNG_LENGTH,"byteOffset":36}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;

    /// Load the textured triangle with `image` as its base color, and return
    /// the first pixel of the source its Material keeps.
    fn load_texture_pixel(image: image::DynamicImage) -> [u8; 4] {
        let mut png = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode PNG");
        let png = png.into_inner();
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        bin.extend_from_slice(&png);
        let json = TEXTURED_JSON
            .replace("BUFFER_LENGTH", &bin.len().to_string())
            .replace("PNG_LENGTH", &png.len().to_string());
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        let scene = Scene::load(build_glb(&json, &bin)).expect("load textured triangle");
        let models = scene.models();
        let maps = models[0].material().shader().object.maps.read();
        assert_eq!(maps.len(), 1, "the base color texture should be a map");
        match &*maps[0].source.data {
            SourceData::Bytes(bytes) => [bytes[0], bytes[1], bytes[2], bytes[3]],
            other => panic!("expected decoded bytes, got {other:?}"),
        }
    }

    /// The base color material of the textured triangle, which the UV tests replace.
    const TEXTURED_MATERIAL: &str =
        r#""materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}]"#;

    /// Load the textured triangle with `materials` in place of its own,
    /// every texture reading one 2×2 PNG.
    fn load_textured(materials: &str) -> Result<Scene, SceneLoadError> {
        let white = image::ImageBuffer::from_pixel(2, 2, image::Rgba([255u8; 4]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(white)
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode PNG");
        let png = png.into_inner();
        #[rustfmt::skip]
        let positions: [f32; 9] = [
             0.0,  0.5, 0.0,
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
        ];
        let mut bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        bin.extend_from_slice(&png);
        assert!(TEXTURED_JSON.contains(TEXTURED_MATERIAL));
        let json = TEXTURED_JSON
            .replace(TEXTURED_MATERIAL, materials)
            .replace("BUFFER_LENGTH", &bin.len().to_string())
            .replace("PNG_LENGTH", &png.len().to_string());
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        Scene::load(build_glb(&json, &bin))
    }

    /// The UV set and transform of the material slot at `index`.
    fn read_slot(material: &Material, index: usize) -> (u32, [f32; 2], [f32; 2], f32) {
        let read = |field: &str| format!("material.uv[{index}].{field}");
        let shader = material.shader();
        (
            shader.get(&read("uv_set")).expect("uv_set"),
            shader.get(&read("offset")).expect("offset"),
            shader.get(&read("scale")).expect("scale"),
            shader.get(&read("rotation")).expect("rotation"),
        )
    }

    /// JSON written with single quotes, so each line stays one Rust string.
    fn quote(lines: &[&str]) -> String {
        lines.concat().replace('\'', "\"")
    }

    // Story: an ORM image that the metallic-roughness and occlusion slots both read
    // decodes once, and the two maps share its pixels.
    #[test]
    fn two_slots_that_read_one_image_share_its_pixels() {
        let materials = quote(&[
            "'materials':[{'pbrMetallicRoughness':{'metallicRoughnessTexture':{'index':0}},",
            "'occlusionTexture':{'index':0}}]",
        ]);
        let scene = load_textured(&materials).expect("load");
        let models = scene.models();
        let maps = models[0].material().shader().object.maps.read();
        let read = |key: &str| {
            let map = maps.iter().find(|map| map.key == key);
            map.map(|map| map.source.data.clone()).expect("a map")
        };

        let roughness = read("metallic_roughness_map");
        let occlusion = read("occlusion_map");

        assert!(
            Arc::ptr_eq(&roughness, &occlusion),
            "each slot decoded a copy of its own"
        );
    }

    #[test]
    fn load_reads_each_texture_coord_and_transform() {
        // Base color and occlusion carry different transforms. Normal moves to
        // set 1 and emissive back to set 0 through the extension's texCoord.
        let materials = quote(&[
            "'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0,",
            "'extensions':{'KHR_texture_transform':{'offset':[0.5,0.0],'scale':[2.0,2.0],'rotation':0.25}}}},",
            "'occlusionTexture':{'index':0,'texCoord':1,",
            "'extensions':{'KHR_texture_transform':{'offset':[0.0,0.25],'scale':[4.0,4.0],'rotation':1.0}}},",
            "'normalTexture':{'index':0,'extensions':{'KHR_texture_transform':{'texCoord':1}}},",
            "'emissiveTexture':{'index':0,'texCoord':1,'extensions':{'KHR_texture_transform':{'texCoord':0}}}}]",
        ]);
        let scene = load_textured(&materials).expect("load");
        let models = scene.models();
        let material = models[0].material();
        assert_eq!(read_slot(material, 0), (0, [0.5, 0.0], [2.0, 2.0], 0.25));
        assert_eq!(read_slot(material, 1), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
        assert_eq!(read_slot(material, 2), (1, [0.0, 0.0], [1.0, 1.0], 0.0));
        assert_eq!(read_slot(material, 3), (1, [0.0, 0.25], [4.0, 4.0], 1.0));
        assert_eq!(read_slot(material, 4), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
    }

    #[test]
    fn load_reads_a_null_texture_transform_as_none_on_every_slot() {
        // A null KHR_texture_transform is no transform on any slot.
        let materials = quote(&[
            "'materials':[{'pbrMetallicRoughness':{",
            "'baseColorTexture':{'index':0,'extensions':{'KHR_texture_transform':null}},",
            "'metallicRoughnessTexture':{'index':0,'extensions':{'KHR_texture_transform':null}}},",
            "'normalTexture':{'index':0,'texCoord':1,'extensions':{'KHR_texture_transform':null}},",
            "'occlusionTexture':{'index':0,'extensions':{'KHR_texture_transform':null}},",
            "'emissiveTexture':{'index':0,'extensions':{'KHR_texture_transform':null}}}]",
        ]);
        let scene = load_textured(&materials).expect("a null transform loads");
        let models = scene.models();
        let material = models[0].material();
        for index in [0, 1, 3, 4] {
            assert_eq!(read_slot(material, index), (0, [0.0, 0.0], [1.0, 1.0], 0.0));
        }
        assert_eq!(read_slot(material, 2), (1, [0.0, 0.0], [1.0, 1.0], 0.0));
    }

    #[test]
    fn load_refuses_a_third_texture_coord() {
        let occlusion = quote(&["'materials':[{'occlusionTexture':{'index':0,'texCoord':2}}]"]);
        match load_textured(&occlusion) {
            Err(SceneLoadError::Invalid(message)) => {
                assert!(
                    message.contains("occlusionTexture reads TEXCOORD_2"),
                    "{message}"
                );
            }
            other => panic!("expected SceneLoadError::Invalid, got {other:?}"),
        }
        let base = quote(&[
            "'materials':[{'pbrMetallicRoughness':{'baseColorTexture':{'index':0,",
            "'extensions':{'KHR_texture_transform':{'texCoord':3}}}}}]",
        ]);
        match load_textured(&base) {
            Err(SceneLoadError::Invalid(message)) => {
                assert!(
                    message.contains("baseColorTexture reads TEXCOORD_3"),
                    "{message}"
                );
            }
            other => panic!("expected SceneLoadError::Invalid, got {other:?}"),
        }
    }

    /// Load the triangle with `attributes` and the 0.5 pairs of view 2 and
    /// the position floats of view 0 as two VEC2 accessors, 1 and 2.
    /// Returns the UV0 and UV1 of each vertex.
    fn load_vertex_uvs(attributes: &str) -> Vec<(VertexData, VertexData)> {
        let bytes = mutate_triangle(&[
            (ATTRIBUTES, attributes),
            (
                LAST_ACCESSOR,
                r#""max":[0.5,0.5,0.0]},{"bufferView":2,"componentType":5126,"count":3,"type":"VEC2"},{"bufferView":0,"componentType":5126,"count":3,"type":"VEC2"}]"#,
            ),
        ]);
        let scene = Scene::load(bytes).expect("load");
        let models = scene.models();
        let verts = models[0].mesh().object.verts.read();
        verts
            .iter()
            .map(|vertex| {
                let read = |key: &str| vertex.properties.get(key).copied().expect(key);
                (read(crate::Vertex::UV0), read(crate::Vertex::UV1))
            })
            .collect()
    }

    #[test]
    fn load_reads_texcoord_1_into_uv1() {
        let uvs = load_vertex_uvs(r#"{"POSITION":0,"TEXCOORD_0":1,"TEXCOORD_1":2}"#);
        let uv1: Vec<VertexData> = uvs.iter().map(|(_, uv1)| *uv1).collect();
        assert_eq!(
            uv1,
            [
                VertexData::F32x2([0.0, 0.5]),
                VertexData::F32x2([0.0, -0.5]),
                VertexData::F32x2([-0.5, 0.0]),
            ]
        );
        assert!(
            uvs.iter()
                .all(|(uv0, _)| *uv0 == VertexData::F32x2([0.5, 0.5]))
        );
    }

    #[test]
    fn load_gives_uv0_to_uv1_without_texcoord_1() {
        let uvs = load_vertex_uvs(r#"{"POSITION":0,"TEXCOORD_0":2}"#);
        for (uv0, uv1) in uvs {
            assert_eq!(uv1, uv0);
            assert_ne!(
                uv0,
                VertexData::F32x2([0.0, 0.0]),
                "UV0 comes from the file"
            );
        }
    }

    #[test]
    fn load_decodes_a_16_bit_gray_png() {
        let white = image::ImageBuffer::from_pixel(2, 2, image::Luma([u16::MAX]));
        let pixel = load_texture_pixel(image::DynamicImage::ImageLuma16(white));
        assert_eq!(pixel, [255, 255, 255, 255]);
    }

    #[test]
    fn load_decodes_a_gray_alpha_png() {
        let gray = image::ImageBuffer::from_pixel(2, 2, image::LumaA([128u8, 64]));
        let pixel = load_texture_pixel(image::DynamicImage::ImageLumaA8(gray));
        assert_eq!(pixel, [128, 128, 128, 64]);
    }

    #[test]
    fn decode_image_refuses_short_pixels() {
        let image = gltf::image::Data {
            pixels: vec![0; 6],
            format: gltf::image::Format::R16,
            width: 2,
            height: 2,
        };
        let error = decode_image(&image).expect_err("6 bytes can't hold 4 pixels of R16");
        assert!(error.to_string().contains("R16 image has fewer bytes"));
    }

    #[test]
    fn compute_vertex_normals_indexed_yz_face() {
        // Triangle in the YZ plane. Face normal = +X. Without
        // face-normal computation the fallback +Z would shade this
        // triangle as if it pointed forward — visibly wrong.
        let positions = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let indices = [0u32, 1, 2];
        let normals = compute_vertex_normals("test", &positions, Some(&indices)).expect("normals");
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
        let normals = compute_vertex_normals("test", &positions, None).expect("normals");
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
        let normals = compute_vertex_normals("test", &positions, Some(&indices)).expect("normals");
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
        assert!(
            !opts.smooth,
            "magFilter=9728 (NEAREST) should set smooth=false"
        );
    }
}
