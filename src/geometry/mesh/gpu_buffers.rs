//! The GPU buffers a mesh uploads for one `RenderContext` at a time,
//! and the upload it caches for the context that drew it last.

use std::fmt::Debug;
use std::sync::Arc;

use super::MeshObject;
use super::packing::{Repacked, Schemas, count};
use super::schema::AttributeTypes;
use crate::RendererError;
use crate::core::{RenderContext, VertexLayouts, check_limit};
use crate::geometry::MeshError;

/// What one attached shader reads from the mesh, and the
/// vertex layouts it read, with the schemas of the pack they came from.
#[derive(Debug, Default)]
pub(super) struct Attached {
    pub(super) reads: AttributeTypes,
    pub(super) layouts: Option<(Arc<Schemas>, VertexLayouts)>,
}

/// An upload and the context whose device holds its buffers,
/// in one value, so an upload never reaches another context.
#[derive(Debug)]
pub(super) struct Cached {
    pub(super) owner: u64,
    pub(super) upload: Upload,
}

/// The buffers an upload wrote, the counts that draw them and the layouts their
/// bytes follow, all from one pack, with the mesh's generation when the upload began.
#[derive(Clone, Debug)]
pub(crate) struct Upload {
    pub(crate) buffers: VertexBuffers,
    pub(crate) counts: DrawCounts,
    pub(crate) schemas: Arc<Schemas>,
    pub(crate) generation: u64,
}

/// A mesh's buffers on the device of one `Renderer`. Another `Renderer`
/// that draws the mesh uploads buffers of its own in their place.
#[derive(Debug)]
pub(super) struct GpuStreams {
    /// The `RenderContext::id` of the context whose device holds the buffers.
    pub(super) owner: u64,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: u64,
}

impl MeshObject {
    /// Get Vertex Buffers, Draw counts and the layouts they follow for this mesh,
    /// creating or updating GPU resources on the device of `context` as needed.
    /// Buffers on another context's device give way to new ones.
    pub(crate) fn upload(&self, context: &RenderContext) -> Result<Upload, RendererError> {
        let generation = self.generation();
        #[cfg(test)]
        crate::testing::race_hooks::run("upload");
        if let Some(found) = self.find_cached(context.id, generation) {
            return Ok(found);
        }
        // One upload at a time, and the result it caches names the context it uploaded for.
        let mut gpu = self.gpu.write();
        if let Some(found) = self.find_cached(context.id, generation) {
            return Ok(found);
        }
        let upload = self.upload_vertex_buffers(context, &mut gpu, generation)?;
        *self.cache.write() = Some(Cached {
            owner: context.id,
            upload: upload.clone(),
        });
        Ok(upload)
    }

    /// The cached upload for the context `owner`, unless the mesh changed since `generation`.
    pub(super) fn find_cached(&self, owner: u64, generation: u64) -> Option<Upload> {
        let cache = self.cache.read();
        let cached = cache.as_ref()?;
        let current = cached.owner == owner && cached.upload.generation == generation;
        current.then(|| cached.upload.clone())
    }

    /// Packs and uploads the mesh into `gpu`. An upload that fails to write a stream
    /// it packed leaves that stream unpacked, so the next one packs and writes it again.
    fn upload_vertex_buffers(
        &self,
        context: &RenderContext,
        gpu: &mut Option<GpuStreams>,
        generation: u64,
    ) -> Result<Upload, RendererError> {
        let repacked = self.ensure_packed()?;
        #[cfg(test)]
        crate::testing::race_hooks::run("packed");
        let uploaded = self.write_buffers(context, gpu, repacked, generation);
        if uploaded.is_err() {
            self.packed.write().forget(repacked);
        }
        uploaded
    }

    /// Writes what the last pack made into `gpu`, the streams `repacked` names at least.
    fn write_buffers(
        &self,
        context: &RenderContext,
        gpu: &mut Option<GpuStreams>,
        repacked: Repacked,
        generation: u64,
    ) -> Result<Upload, RendererError> {
        let (device, queue) = (&context.device, &context.queue);

        let packed = self.packed.read();
        let (vertices, instances, indices) = (&packed.vertices, &packed.instances, &packed.indices);

        // The rows come from the pack that made the bytes, so instances without
        // properties count, and a list that shrinks into the old buffer draws its own length.
        let instance_rows = match packed.rows {
            0 => None,
            rows => Some(count(rows)?),
        };
        let index_count = count(indices.len())?;

        // Checked on every upload: a failed one leaves the streams dirty,
        // so data that stays too large fails again instead of drawing old buffers.
        let limit = context.limits.max_buffer_size;
        let sizes = [
            ("the mesh's vertex buffer", vertices.len() as u64),
            ("the mesh's index buffer", u64::from(index_count) * 4),
            ("the mesh's instance buffer", instances.len() as u64),
        ];
        for (buffer, size) in sizes {
            check_limit(buffer, "max_buffer_size", size, limit)?;
        }

        // Decide whether we need to recreate GPU buffers. Buffers on
        // another device can't draw here, so they give way to new ones.
        let mut need_new = match gpu.as_ref() {
            None => true,
            Some(streams) => {
                streams.owner != context.id
                    || streams.index_count != index_count
                    || repacked.vertices
            }
        };

        if let Some(streams) = gpu.as_ref() {
            // Instance buffer presence/size changes force (re)creation
            let had_instances = streams.instance_buffer.is_some();
            let wants_instances = !instances.is_empty();
            if had_instances != wants_instances {
                need_new = true;
            } else if wants_instances {
                let capacity = streams.instance_buffer_capacity;
                let needed = instances.len() as u64;
                if needed > capacity {
                    need_new = true;
                }
            }
        }

        if need_new {
            let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Vertex Buffer"),
                size: vertices.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Index Buffer"),
                size: (indices.len() as u64) * 4,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&vertex_buffer, 0, vertices);
            queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(indices));

            let (instance_buffer, instance_bytes) = if !instances.is_empty() {
                let needed = instances.len() as u64;
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Instance Buffer"),
                    size: needed,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buffer, 0, instances);
                (Some(buffer), needed)
            } else {
                (None, 0)
            };

            gpu.replace(GpuStreams {
                owner: context.id,
                vertex_buffer,
                index_buffer,
                index_count,
                instance_buffer,
                instance_buffer_capacity: instance_bytes,
            });
        } else if repacked.instances
            && let Some(buffer) = gpu
                .as_ref()
                .and_then(|streams| streams.instance_buffer.as_ref())
        {
            // The vertices didn't change, or the buffers would be new. The instances
            // fit the buffer they have, as the capacity check above found.
            queue.write_buffer(buffer, 0, instances);
        }

        let Some(streams) = gpu.as_ref() else {
            return Err(MeshError::NoGpuStreams.into());
        };
        let vertex_buffers = VertexBuffers {
            vertex_buffer: streams.vertex_buffer.clone(),
            index_buffer: streams.index_buffer.clone(),
            instance_buffer: streams.instance_buffer.clone(),
        };

        let override_count = *self.override_instances.read();
        let draw_counts = DrawCounts {
            index_count: streams.index_count,
            instance_count: override_count.or(instance_rows).unwrap_or(1),
            instance_rows,
        };

        Ok(Upload {
            buffers: vertex_buffers,
            counts: draw_counts,
            schemas: packed.schemas.clone(),
            generation,
        })
    }
}

pub(crate) struct VertexBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: Option<wgpu::Buffer>,
}

impl Clone for VertexBuffers {
    fn clone(&self) -> Self {
        Self {
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            instance_buffer: self.instance_buffer.clone(),
        }
    }
}

impl Debug for VertexBuffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexBuffers")
            .field("vertex_buffer", &self.vertex_buffer)
            .field("index_buffer", &self.index_buffer)
            .field("instance_buffer", &self.instance_buffer)
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DrawCounts {
    pub index_count: u32,
    pub instance_count: u32,
    /// The instances the mesh holds, or `None` when it holds none.
    pub instance_rows: Option<u32>,
}
