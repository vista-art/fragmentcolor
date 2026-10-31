//! Reads each glTF primitive into a `Mesh`: checks its accessors,
//! expands strips and fans into triangle lists, and fills in missing normals.

use glam::Vec3;
use gltf::Semantic;
use gltf::accessor::{DataType, Dimensions};

use super::SceneLoadError;
use crate::Mesh;

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
pub(super) fn reserve<Item>(count: usize, what: &str) -> Result<Vec<Item>, SceneLoadError> {
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

pub(super) fn build_mesh(
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
    // For an asset without tangents, `[1, 0, 0, 1]` gives the basis
    // T = +X, B = +Y, N = +Z. A normal map reads right on a surface facing +Z
    // and wrong on others, since the loader doesn't compute MikkTSpace tangents.
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
            crate::Vertex::new(*pos)
                .set(crate::Vertex::NORMAL, n)
                .set(crate::Vertex::UV0, uv)
                .set(crate::Vertex::COLOR0, color)
                .set(crate::Vertex::UV1, uv1)
                .set(crate::Vertex::TANGENT, tangent),
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
/// by any triangle) fall back to a forward-Z normal, so the BRDF stays
/// finite, as glTF 2.0 does for a mesh with only POSITION.
pub(super) fn compute_vertex_normals(
    label: &str,
    positions: &[[f32; 3]],
    indices: Option<&[u32]>,
) -> Result<Vec<[f32; 3]>, SceneLoadError> {
    let count = positions.len();
    let mut sums = reserve(count, label)?;
    sums.resize(count, Vec3::ZERO);
    let visit_triangle = |a: usize, b: usize, c: usize, sums: &mut [Vec3]| {
        if a >= count || b >= count || c >= count {
            return;
        }
        let v0 = Vec3::from(positions[a]);
        let v1 = Vec3::from(positions[b]);
        let v2 = Vec3::from(positions[c]);
        let face = (v1 - v0).cross(v2 - v0);
        sums[a] += face;
        sums[b] += face;
        sums[c] += face;
    };
    if let Some(indices) = indices {
        for triangle in indices.as_chunks::<3>().0 {
            visit_triangle(
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
                &mut sums,
            );
        }
    } else {
        let mut index = 0;
        while index + 2 < count {
            visit_triangle(index, index + 1, index + 2, &mut sums);
            index += 3;
        }
    }
    let mut normals = reserve(count, label)?;
    normals.extend(sums.iter().map(|sum| {
        sum.try_normalize()
            .unwrap_or(Vec3::new(0.0, 0.0, 1.0))
            .to_array()
    }));
    Ok(normals)
}
