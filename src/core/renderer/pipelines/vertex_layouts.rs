//! Vertex buffer layouts built from mesh schemas,
//! and the check of their strides against the device.

use crate::ShaderObject;
use crate::core::renderer::{RendererError, check_limit};
use crate::geometry::{MeshObject, POSITION, Schemas, VertexSchema};
use std::collections::HashMap;

/// One vertex buffer slot of a render pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VertexStream {
    pub(in crate::core::renderer) stride: u64,
    step_mode: wgpu::VertexStepMode,
    pub(in crate::core::renderer) attributes: Vec<wgpu::VertexAttribute>,
}

/// Vertex buffer layouts a shader reads from its mesh:
/// slot 0 steps per vertex, the optional slot 1 per instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VertexLayouts {
    vertex: VertexStream,
    pub(in crate::core::renderer) instance: Option<VertexStream>,
}

impl VertexLayouts {
    pub(super) fn buffers(&self) -> Vec<wgpu::VertexBufferLayout<'_>> {
        std::iter::once(&self.vertex)
            .chain(self.instance.as_ref())
            .map(|stream| wgpu::VertexBufferLayout {
                array_stride: stream.stride,
                step_mode: stream.step_mode,
                attributes: &stream.attributes,
            })
            .collect()
    }
}

/// The vertex layouts `shader` reads from `mesh`.
/// Its buffers follow `schemas`, and the mesh keeps the layouts until it packs again
/// (`MeshObject::fetch_layouts`). `check_strides` checks them against each device.
pub(in crate::core::renderer) fn vertex_buffer_layouts(
    shader: &ShaderObject,
    mesh: &MeshObject,
    schemas: &Schemas,
) -> Result<VertexLayouts, RendererError> {
    let vertex_inputs = shader.reflect_vertex_inputs()?;

    let Some(vertex_schema) = schemas.vertex.as_ref() else {
        return Err(RendererError::Error(
            "Mesh attribute mapping failed: no vertex schema".into(),
        ));
    };

    // Precompute name -> (offset, fmt) for both streams
    let (vertex_map, vertex_stride) = schema_offsets(vertex_schema);
    let (instance_map, instance_stride) = if let Some(schema) = schemas.instance.as_ref() {
        let (map, stride) = schema_offsets(schema);
        (Some(map), Some(stride))
    } else {
        (None, None)
    };

    // Partition attributes per stream based on mapping rule
    let mut vertex_attributes: Vec<wgpu::VertexAttribute> = Vec::new();
    let mut instance_attributes: Vec<wgpu::VertexAttribute> = Vec::new();

    // Build optional index->name maps from the first vertex/instance
    let (vertex_location, vertex_locations) = mesh.vertex_location_map();
    let instance_locations = mesh.instance_location_map();

    // Match each shader vertex-input to a mesh attribute. Name match wins
    // over location match: shaders that use fixed `@location(...)` slots
    // (e.g. the PBR shader's `model_0..model_3` at locations 3..6) collide
    // with the vertex's auto-incremented locations once the vertex carries
    // many attributes (NORMAL, UV0, COLOR0, UV1, ...). The mesh's
    // location_map numbers in its own buffer-local space, not the shader's
    // global one, so a shader-location of 4 means "model_1" for the
    // instance buffer but might be "uv1" in the vertex map. Falling back to
    // location-match would then mis-attribute the slot. Name-first kills
    // the ambiguity — `position`, `model_*`, `color0`, etc. resolve by
    // string. Location-based matching catches the remaining case where the
    // shader's input name doesn't appear in either map (raw / generated
    // shaders).
    for vertex_input in vertex_inputs.iter() {
        let mut placed = false;

        let assert_format = |format_mesh: wgpu::VertexFormat,
                             stream: &'static str|
         -> Result<(), RendererError> {
            if vertex_input.format != format_mesh {
                return Err(RendererError::Error(format!(
                    "Type mismatch for shader input '{}' @location({}) ({stream}): shader expects {:?}, mesh has {:?}",
                    vertex_input.name, vertex_input.location, vertex_input.format, format_mesh
                )));
            }
            Ok(())
        };

        // 1) Name match against instance (handles `model_*` at fixed shader
        //    locations regardless of the mesh's instance-side numbering).
        if let Some(ref mi) = instance_map
            && let Some((offset, format_mesh)) = mi.get(vertex_input.name.as_str()).cloned()
        {
            assert_format(format_mesh, "instance")?;
            instance_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        // 2) Name match against vertex (handles NORMAL / UV0 / UV1 /
        //    COLOR0 / TANGENT regardless of insertion order).
        if !placed
            && let Some((offset, format_mesh)) = vertex_map.get(vertex_input.name.as_str()).cloned()
        {
            assert_format(format_mesh, "vertex")?;
            vertex_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        // 3) Position fallback — the only vertex slot that's commonly
        //    unnamed in the mesh map.
        if !placed
            && vertex_input.location == vertex_location
            && let Some((offset, format_mesh)) = vertex_map.get(POSITION).cloned()
        {
            assert_format(format_mesh, "vertex/position")?;
            vertex_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        // 4) Instance match by buffer-local location index (for instance
        //    schemas built without canonical names).
        if !placed
            && let Some(name) = instance_locations.get(&vertex_input.location)
            && let Some((offset, format_mesh)) =
                instance_map.as_ref().and_then(|map| map.get(name)).cloned()
        {
            assert_format(format_mesh, "instance/loc")?;
            instance_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        // 5) Vertex match by buffer-local location index.
        if !placed
            && let Some(name) = vertex_locations.get(&vertex_input.location)
            && let Some((offset, format_mesh)) = vertex_map.get(name.as_str()).cloned()
        {
            assert_format(format_mesh, "vertex/loc")?;
            vertex_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        if !placed {
            return Err(RendererError::Error(format!(
                "Mesh attribute not found for shader input '{}' @location({})",
                vertex_input.name, vertex_input.location
            )));
        }
    }

    vertex_attributes.sort_by_key(|attr| attr.shader_location);
    let vertex = VertexStream {
        stride: vertex_stride,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: vertex_attributes,
    };

    // A pipeline steps the instance stream only when the shader reads it.
    let instance = match (instance_attributes.is_empty(), instance_stride) {
        (false, Some(stride)) => {
            instance_attributes.sort_by_key(|attr| attr.shader_location);
            Some(VertexStream {
                stride,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: instance_attributes,
            })
        }
        _ => None,
    };

    Ok(VertexLayouts { vertex, instance })
}

/// Refuses a vertex stride, or an instance stride the shader reads, past the device's limit.
/// It allocates nothing unless it refuses, so it runs on every frame.
pub(in crate::core::renderer) fn check_strides(
    layouts: &VertexLayouts,
    limits: &wgpu::Limits,
) -> Result<(), RendererError> {
    let limit = "max_vertex_buffer_array_stride";
    let max = u64::from(limits.max_vertex_buffer_array_stride);
    check_limit(
        "the mesh's vertex stride",
        limit,
        layouts.vertex.stride,
        max,
    )?;
    match layouts.instance.as_ref() {
        Some(instance) => check_limit("the mesh's instance stride", limit, instance.stride, max),
        None => Ok(()),
    }
}

fn schema_offsets(schema: &VertexSchema) -> (HashMap<String, (u64, wgpu::VertexFormat)>, u64) {
    let mut map = HashMap::new();
    let mut ofs = 0u64;
    for f in schema.fields.iter() {
        map.insert(f.name.clone(), (ofs, f.fmt));
        ofs += f.size;
    }
    (map, schema.stride)
}
