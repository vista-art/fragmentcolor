//! Scene loader — `Scene::load` and the `SceneSource` transport.
//!
//! `Scene::load` accepts a path or in-memory bytes directly; the input is
//! converted into a format-tagged [`SceneSource`] internally so the loader
//! knows which parser to dispatch. glTF 2.0 (with its `.glb` binary
//! container) is the first format; the enum gains a variant per future
//! format without changing the public method.
//!
//! The shape mirrors `Material::pbr()` and `Camera::perspective(...)`: sync
//! return, no `Renderer` argument. Any texture inputs the parser produces
//! flow into the Material as pending [`crate::TextureInput`] entries and
//! the renderer drains them on first render (see [`crate::Renderer::load`]).
//!
//! The loader covers static glTF: mesh primitives (positions, normals,
//! two UV sets, colors, tangents, indices), PBR-MR materials with all five
//! texture slots, each with the UV set and `KHR_texture_transform` it names,
//! per-node transforms flattened into Model matrices, glTF camera nodes,
//! and `KHR_lights_punctual` lights. Animation, skinning, morph targets,
//! and material extensions beyond PBR-MR are out of scope here.

use std::collections::HashSet;
use std::path::PathBuf;

use glam::{Mat4, Quat, Vec3};
use gltf::Semantic;
use gltf::accessor::{DataType, Dimensions};

use crate::material::{TextureSlot, UvSet, UvTransform};
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
    #[error("glTF camera: {0}. Load with cameras(false) to skip the file's cameras")]
    Camera(#[from] crate::scene::CameraError),
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
                "Scene::load(path) is not supported on wasm32. Fetch the bytes from JS and pass them to Scene::load(bytes)".into(),
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
                &mut material_cache,
                &options,
            )?;
            let first_child = stack.len();
            stack.extend(node.children().map(|child| (child, world)));
            stack[first_child..].reverse();
        }
    }

    Ok(scene)
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
    material_cache: &mut std::collections::HashMap<Option<usize>, Material>,
    options: &GltfOptions,
) -> Result<Mat4, SceneLoadError> {
    let local = local_transform(node);
    let world = parent_world * local;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let fc_mesh = build_mesh(&mesh, &primitive, buffers)?;
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
    if let Some(indices) = primitive.indices() {
        check_accessor(&indices, "indices", INDEX_TYPES, buffers)?;
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
    // A sparse accessor without a view holds zeros that cost no bytes in the
    // file, so nothing else bounds its count. Past u32::MAX no index reaches it.
    if count > u32::MAX as usize {
        return Err(SceneLoadError::Invalid(format!(
            "{label} holds {count} elements, more than 32-bit indices reach"
        )));
    }
    let size = accessor.size();
    if count.checked_mul(size).is_none() {
        return Err(SceneLoadError::Invalid(format!(
            "{label} holds {count} elements of {size} bytes, more than this platform addresses"
        )));
    }
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

/// Collect a declared accessor's items, or `None` for an absent one.
///
/// The reader returns `None` both for an absent attribute and for one whose
/// bytes it can't reach, and only the first case may fall back to defaults.
/// The memory is reserved up front and fallibly, so a count too large to hold
/// is an error instead of an allocation panic.
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
    let mut values = Vec::new();
    if values.try_reserve_exact(accessor.count()).is_err() {
        return Err(SceneLoadError::Invalid(format!(
            "glTF accessor {} ({semantic}) holds {} elements, more than fit in memory",
            accessor.index(),
            accessor.count()
        )));
    }
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
            (0..count).collect()
        }
    };
    Ok(Some(expand(&order)))
}

/// One triangle per step of a strip, every second one flipped back to the
/// strip's winding.
fn expand_strip(order: &[u32]) -> Vec<u32> {
    order
        .windows(3)
        .enumerate()
        .flat_map(|(step, corners)| match step % 2 {
            0 => [corners[0], corners[1], corners[2]],
            _ => [corners[1], corners[0], corners[2]],
        })
        .collect()
}

/// One triangle per step of a fan, each sharing the first vertex.
fn expand_fan(order: &[u32]) -> Vec<u32> {
    let Some((&hub, rim)) = order.split_first() else {
        return Vec::new();
    };
    rim.windows(2)
        .flat_map(|edge| [hub, edge[0], edge[1]])
        .collect()
}

/// Refuse an index that points past the last vertex.
fn check_indices(
    label: &str,
    indices: Option<&[u32]>,
    vertex_count: usize,
) -> Result<(), SceneLoadError> {
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

    if let Some(info) = pbr.base_color_texture() {
        material = material.base_color_texture(convert_image(&info.texture(), images)?);
        apply_uv(&material, TextureSlot::BaseColor, read_info_uv(&info))?;
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        material = material.metallic_roughness_texture(convert_image(&info.texture(), images)?);
        apply_uv(
            &material,
            TextureSlot::MetallicRoughness,
            read_info_uv(&info),
        )?;
    }
    if let Some(info) = gltf_material.normal_texture() {
        material = material.normal_texture(convert_image(&info.texture(), images)?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Normal, uv)?;
    }
    if let Some(info) = gltf_material.occlusion_texture() {
        material = material.occlusion_texture(convert_image(&info.texture(), images)?);
        let extension = info.extension_value(TEXTURE_TRANSFORM);
        let uv = read_extension_uv(info.tex_coord(), extension)?;
        apply_uv(&material, TextureSlot::Occlusion, uv)?;
    }
    if let Some(info) = gltf_material.emissive_texture() {
        material = material.emissive_texture(convert_image(&info.texture(), images)?);
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
/// keeps their `KHR_texture_transform` as raw JSON, parsed here.
fn read_extension_uv(
    tex_coord: u32,
    extension: Option<&gltf::json::Value>,
) -> Result<TextureUv, SceneLoadError> {
    use gltf::json::extensions::texture::TextureTransform;
    let Some(value) = extension else {
        return Ok(TextureUv {
            tex_coord,
            transform: UvTransform::default(),
        });
    };
    let transform: TextureTransform = gltf::json::deserialize::from_value(value.clone())
        .map_err(|error| SceneLoadError::Invalid(format!("{TEXTURE_TRANSFORM}: {error}")))?;
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

/// Convert a glTF image reference into the TextureInput the lazy Material
/// setter queues. Every pixel format the importer decodes converts to RGBA8,
/// and an image with fewer bytes than its size needs is an error.
fn convert_image(
    texture: &gltf::Texture<'_>,
    images: &[gltf::image::Data],
) -> Result<TextureInput, SceneLoadError> {
    let image = images
        .get(texture.source().index())
        .ok_or_else(|| SceneLoadError::Invalid("glTF texture references missing image".into()))?;
    let rgba = decode_image(image)?;
    let sampler_options = map_sampler_options(&texture.sampler());
    Ok(TextureInput {
        data: TextureData::Bytes(rgba.into_raw()),
        options: crate::TextureOptions {
            size: Some(crate::Size::from([image.width, image.height])),
            sampler: sampler_options,
            ..Default::default()
        },
    })
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

/// Lift a glTF camera node into a FragmentColor [`Camera`]. The node's
/// world matrix gives the eye position + view orientation; the camera's
/// projection comes from the glTF camera projection block.
fn build_camera(gltf_camera: &gltf::Camera<'_>, world: Mat4) -> Result<Camera, SceneLoadError> {
    let (near, far) = match gltf_camera.projection() {
        // glTF's zfar is optional for a perspective camera; map "infinite"
        // to a large finite value the depth buffer can still represent.
        gltf::camera::Projection::Perspective(p) => (p.znear(), p.zfar().unwrap_or(1.0e6)),
        gltf::camera::Projection::Orthographic(o) => (o.znear(), o.zfar()),
    };
    // A Camera takes far < near as reverse-Z, but glTF requires zfar > znear.
    if far <= near {
        return Err(SceneLoadError::Invalid(format!(
            "glTF camera {} has zfar {far} at or before znear {near}",
            gltf_camera.index()
        )));
    }
    let camera = match gltf_camera.projection() {
        gltf::camera::Projection::Perspective(p) => {
            // glTF's aspectRatio is optional; fall back to 1.0 (a common
            // default that gets overridden when the user retargets to a
            // specific render target).
            let aspect = p.aspect_ratio().unwrap_or(1.0);
            Camera::perspective(p.yfov(), aspect, near, far)?
        }
        gltf::camera::Projection::Orthographic(o) => {
            let half_w = o.xmag();
            let half_h = o.ymag();
            Camera::orthographic(-half_w, half_w, -half_h, half_h, near, far)?
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
    Ok(camera.look_at(eye, target_v.into(), up.into())?)
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

    const DIRECTIONAL_LIGHT: &str =
        r#"{"type":"directional","color":[1.0,0.95,0.9],"intensity":2.0}"#;

    #[test]
    fn load_refuses_a_negative_light_range() {
        let json = CAMERA_LIGHT_JSON.replace(DIRECTIONAL_LIGHT, r#"{"type":"point","range":-5.0}"#);
        assert_invalid(
            build_camera_light_glb(&json),
            "glTF light 0: Light::set_range expects a non-negative number, got -5",
        );
    }

    /// Load a variant of the camera file and expect `SceneLoadError::Camera`.
    fn assert_camera_error(json: &str) {
        match Scene::load(build_camera_light_glb(json)) {
            Err(SceneLoadError::Camera(error)) => {
                assert!(error.to_string().contains("Camera::"), "{error}")
            }
            Err(other) => panic!("expected SceneLoadError::Camera, got {other:?}"),
            Ok(_) => panic!("expected SceneLoadError::Camera, got a Scene"),
        }
    }

    #[test]
    fn load_refuses_a_camera_with_no_field_of_view() {
        assert_camera_error(&CAMERA_LIGHT_JSON.replace(r#""yfov":1.0"#, r#""yfov":0.0"#));
    }

    #[test]
    fn load_refuses_a_camera_node_with_zero_scale() {
        let json =
            CAMERA_LIGHT_JSON.replace(r#"{"camera":0}"#, r#"{"camera":0,"scale":[0.0,0.0,0.0]}"#);
        assert_camera_error(&json);
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
    /// the first pixel of the texture its Material queued.
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
        let pending = models[0].material().shader().object.pending_textures.read();
        assert_eq!(pending.len(), 1, "the base color texture should be queued");
        match &pending[0].input.data {
            TextureData::Bytes(bytes) => [bytes[0], bytes[1], bytes[2], bytes[3]],
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
        assert!(
            !opts.smooth,
            "magFilter=9728 (NEAREST) should set smooth=false"
        );
    }
}
