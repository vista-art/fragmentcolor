//! Packs vertices and instances into bytes, and checks counts and indices.

use std::collections::HashMap;
use std::sync::Arc;

use super::schema::{Field, VertexSchema};
use crate::geometry::{Instance, MeshError, POSITION, Vertex, describe, explain_position};

/// The bytes and indices the last pack made, with the change counts they hold.
#[derive(Debug, Default)]
pub(super) struct Packed {
    /// The unique vertices in the mesh layout, or every vertex in order with `set_indices`.
    pub(super) vertices: Vec<u8>,
    pub(super) instances: Vec<u8>,
    /// The checked list the mesh draws: derived from the vertices, or the one `set_indices` gave.
    pub(super) indices: Vec<u32>,
    /// The instances `instances` holds, those without properties included.
    pub(super) rows: usize,
    /// The layouts `vertices` and `instances` follow. A repack while an upload or a kept layout
    /// holds them makes new ones, so `Arc::ptr_eq` tells two packs apart.
    pub(super) schemas: Arc<Schemas>,
    /// The `vertex_changes` and `instance_changes` the bytes come from, `None` until packed.
    pub(super) vertices_at: Option<u64>,
    pub(super) instances_at: Option<u64>,
}

/// The vertex and instance layouts the bytes of one pack follow.
#[derive(Clone, Debug, Default)]
pub(crate) struct Schemas {
    pub(crate) vertex: Option<VertexSchema>,
    pub(crate) instance: Option<VertexSchema>,
}

impl Packed {
    /// Marks the streams `repacked` names unpacked, after an upload that failed to write them.
    pub(super) fn forget(&mut self, repacked: Repacked) {
        if repacked.vertices {
            self.vertices_at = None;
        }
        if repacked.instances {
            self.instances_at = None;
        }
    }
}

/// The streams a pack packed again.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Repacked {
    pub(super) vertices: bool,
    pub(super) instances: bool,
}

/// Packs vertex `index` in the mesh layout `schema`. A field an attached shader reads
/// (`required`) must hold a value, and any other field the vertex leaves out packs as zeros.
pub(super) fn pack_vertex(
    out: &mut Vec<u8>,
    index: usize,
    v: &Vertex,
    schema: &VertexSchema,
    required: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), MeshError> {
    if v.properties.contains_key(POSITION) {
        return Err(MeshError::InvalidVertex(explain_position(index)));
    }
    for f in schema.fields.iter() {
        match f.name.as_str() {
            POSITION => {
                let [x, y, z, _] = v.position.0.to_array();
                match (v.dimensions, f.fmt) {
                    (0..=2, wgpu::VertexFormat::Float32x2) => {
                        out.extend_from_slice(bytemuck::cast_slice(&[x, y]));
                    }
                    (3.., wgpu::VertexFormat::Float32x3) => {
                        out.extend_from_slice(bytemuck::cast_slice(&[x, y, z]));
                    }
                    (dimensions, format) => {
                        return Err(MeshError::LayoutMismatch(format!(
                            "a vertex has a {dimensions}D position, but the mesh layout has {format:?}"
                        )));
                    }
                }
            }
            name => match v.properties.get(name) {
                Some(value) => value.write(f.fmt, out).map_err(|misfit| {
                    MeshError::InvalidVertex(misfit.explain(name, &format!("vertex {index}")))
                })?,
                None if required.contains_key(name) => {
                    let place = format!("vertex {index}");
                    return Err(MeshError::InvalidVertex(explain_missing(f, &place)));
                }
                None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
            },
        }
    }
    Ok(())
}

/// Packs instance `index` in the instance layout `schema`, as `pack_vertex` packs a vertex.
pub(super) fn pack_instance(
    out: &mut Vec<u8>,
    index: usize,
    i: &Instance,
    schema: &VertexSchema,
    required: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        match i.properties.get(&f.name) {
            Some(value) => value.write(f.fmt, out).map_err(|misfit| {
                MeshError::InvalidInstance(misfit.explain(&f.name, &format!("instance {index}")))
            })?,
            None if required.contains_key(&f.name) => {
                let place = format!("instance {index}");
                return Err(MeshError::InvalidInstance(explain_missing(f, &place)));
            }
            None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
        }
    }
    Ok(())
}

/// Why the item at `place` can't leave out `field`, worded as the attach words it.
fn explain_missing(field: &Field, place: &str) -> String {
    format!(
        "{}: expected {}, got no value at {place}",
        field.name,
        describe(field.fmt)
    )
}

/// Refuses instances that can't hold an attribute a shader reads from them: one missing
/// from the instance layout, which comes from the first instance or a Model's columns.
pub(super) fn check_layout(
    required: &HashMap<String, wgpu::VertexFormat>,
    fields: &[Field],
    instances: usize,
) -> Result<(), MeshError> {
    let missing = required
        .iter()
        .filter(|(name, _)| !fields.iter().any(|field| &field.name == *name))
        .min_by_key(|(name, _)| name.as_str());
    let Some((name, format)) = missing else {
        return Ok(());
    };
    let found = match instances {
        0 => "no instances".to_string(),
        _ => "no value at instance 0".to_string(),
    };
    Err(MeshError::InvalidInstance(format!(
        "{name}: expected {}, got {found}",
        describe(*format)
    )))
}

/// Refuses a mesh without `set_indices` whose vertices, drawn in threes,
/// end partway through a triangle.
pub(super) fn check_vertices(vertices: usize) -> Result<(), MeshError> {
    match vertices.is_multiple_of(3) {
        true => Ok(()),
        false => Err(MeshError::VertexCount(vertices)),
    }
}

/// Refuses an index list that ends partway through a triangle,
/// or that points past the last of `vertices`.
pub(super) fn check_indices(indices: &[u32], vertices: usize) -> Result<(), MeshError> {
    if !indices.len().is_multiple_of(3) {
        return Err(MeshError::IndexCount(indices.len()));
    }
    match indices.iter().find(|index| **index as usize >= vertices) {
        Some(index) => Err(MeshError::IndexRange {
            index: *index,
            vertices,
        }),
        None => Ok(()),
    }
}

/// `length` as a 32-bit count, the width the GPU draws indices and instances with.
pub(super) fn count(length: usize) -> Result<u32, MeshError> {
    Ok(u32::try_from(length)?)
}
