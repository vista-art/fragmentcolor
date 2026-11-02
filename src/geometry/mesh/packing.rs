//! Packs vertices and instances into bytes, and checks counts and indices.

use std::collections::HashMap;
use std::sync::Arc;

use super::schema::{Field, Read, Reads, VertexSchema};
use crate::errors::show_number;
use crate::geometry::explain_position;
use crate::geometry::{Instance, MeshError, POSITION, Vertex, VertexData, check_halves};

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

/// Packs vertex `index` in the mesh layout `schema`.
/// A field an attached shader reads (`required`) must hold a value,
/// which fits `f16` when the shader reads it as `f16`, the position too.
/// Every other field the vertex leaves out packs as zeros, as no shader reads it.
pub(super) fn pack_vertex(
    out: &mut Vec<u8>,
    index: usize,
    v: &Vertex,
    schema: &VertexSchema,
    required: &Reads,
) -> Result<(), MeshError> {
    if v.properties.contains_key(POSITION) {
        return Err(MeshError::InvalidVertex(explain_position(index)));
    }
    let place = format!("vertex {index}");
    for f in schema.fields.iter() {
        match f.name.as_str() {
            POSITION => {
                let [x, y, z, _] = v.position.0.to_array();
                let components = match v.dimensions {
                    0..=2 => vec![x, y],
                    _ => vec![x, y, z],
                };
                if let Some(read) = required.position.filter(|read| read.half)
                    && let Err(number) = check_halves(components.into_iter().map(f64::from))
                {
                    let found = show_number(number);
                    let text = format!(
                        "{POSITION}: expected {}, got {found} at {place}",
                        read.declared
                    );
                    return Err(MeshError::InvalidVertex(text));
                }
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
            name => match (v.properties.get(name), required.vertex.get(name)) {
                (Some(value), read) => write_value(out, name, value, f.fmt, read, &place)
                    .map_err(MeshError::InvalidVertex)?,
                (None, Some(read)) => {
                    return Err(MeshError::InvalidVertex(explain_missing(
                        name, read, &place,
                    )));
                }
                (None, None) => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
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
    required: &HashMap<String, Read>,
) -> Result<(), MeshError> {
    let place = format!("instance {index}");
    for f in schema.fields.iter() {
        match (i.properties.get(&f.name), required.get(&f.name)) {
            (Some(value), read) => write_value(out, &f.name, value, f.fmt, read, &place)
                .map_err(MeshError::InvalidInstance)?,
            (None, Some(read)) => {
                let text = explain_missing(&f.name, read, &place);
                return Err(MeshError::InvalidInstance(text));
            }
            (None, None) => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
        }
    }
    Ok(())
}

/// Appends `value` in `format` to `out`, or says why it doesn't fit, worded as the attach
/// words it. A value an `f16` input reads (`read`) must also stay finite after rounding to `f16`.
fn write_value(
    out: &mut Vec<u8>,
    name: &str,
    value: &VertexData,
    format: wgpu::VertexFormat,
    read: Option<&Read>,
    place: &str,
) -> Result<(), String> {
    if let Some(read) = read.filter(|read| read.half)
        && let Err(number) = value.check_half()
    {
        let found = show_number(number);
        return Err(format!(
            "{name}: expected {}, got {found} at {place}",
            read.declared
        ));
    }
    value.write(format, out).map_err(|misfit| match read {
        Some(read) => format!(
            "{name}: expected {}, got {} at {place}",
            read.declared, misfit.found
        ),
        None => misfit.explain(name, place),
    })
}

/// Why the item at `place` can't leave out the attribute `name`, worded as the attach words it.
fn explain_missing(name: &str, read: &Read, place: &str) -> String {
    format!(
        "{name}: expected {}, got no value at {place}",
        read.declared
    )
}

/// Refuses instances that can't hold an attribute a shader reads from them: one missing
/// from the instance layout, which comes from the first instance or a Model's columns.
pub(super) fn check_layout(
    required: &HashMap<String, Read>,
    fields: &[Field],
    instances: usize,
) -> Result<(), MeshError> {
    let missing = required
        .iter()
        .filter(|(name, _)| !fields.iter().any(|field| &field.name == *name))
        .min_by_key(|(name, _)| name.as_str());
    let Some((name, read)) = missing else {
        return Ok(());
    };
    let found = match instances {
        0 => "no instances".to_string(),
        _ => "no value at instance 0".to_string(),
    };
    Err(MeshError::InvalidInstance(format!(
        "{name}: expected {}, got {found}",
        read.declared
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
