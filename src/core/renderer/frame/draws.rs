//! What each pass draws, planned before the frame is acquired.

use crate::core::renderer::checks::pass::{Attachment, PassTargets};
use crate::core::renderer::pipelines::vertex_layouts::{VertexLayouts, VertexStream};
use crate::core::renderer::pipelines::vertex_layouts::{check_strides, vertex_buffer_layouts};
use crate::core::renderer::{RenderContext, RendererError, check_limit};
use crate::geometry::{DrawCounts, MeshError, MeshObject, Upload, VertexBuffers};
use crate::{AlphaMode, PassObject, ShaderError, ShaderObject};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// One translucent draw to issue inside the blend phase of a pass. The
/// renderer builds these from `Pass::model_entries` whose Material declares
/// `alpha_mode: Blend`, sorts them back-to-front by `eye_z`, then walks the
/// sorted list reusing each shader's already-built pipeline + bind groups
/// with a fresh single-instance vertex buffer per draw. Per-entry buffers
/// kill the instance-batching for blend draws (which is unavoidable when
/// over-blending needs strict depth ordering) but every other allocation
/// stays cached across draws.
#[derive(Debug)]
pub(super) struct BlendDraw {
    /// Pointer identity of the shader Arc. Same key the opaque batching
    /// uses, looked up by the blend-phase loop to find the cached
    /// pipeline + bind groups (filed under this key in `process_render_pass`).
    pub(super) shader_ptr: usize,
    /// Strong handle to the mesh so we can fetch its vertex + index buffers
    /// when the draw runs. Also keeps the matching `MeshObject` alive
    /// across the render pass, so the renderer's pipeline + vertex-buffer
    /// lookups (keyed by the underlying pointer) stay valid.
    pub(super) mesh: Arc<MeshObject>,
    /// World matrix snapshot taken at queue-build time. Same value used to
    /// compute `eye_z` — keeping them in sync per draw is what makes the
    /// "live transform" semantics work correctly across the blend phase.
    transform: glam::Mat4,
    /// Eye-space Z of the model origin: `(view * transform * vec4(0,0,0,1)).z`.
    /// Sort comparator field — smaller is farther from the camera (the
    /// camera looks down -Z in right-handed view space), so we sort
    /// ascending and draw farthest-first.
    eye_z: f32,
    /// Single-instance vertex buffer holding this entry's `transform` as
    /// four `vec4<f32>` columns. Populated lazily in `build_pass_draws`
    /// after the sort lands so the buffers don't churn during the sort
    /// compare callback.
    pub(super) instance_buffer: Option<wgpu::Buffer>,
}

/// Output of `Renderer::build_pass_draws`. Splits the pass's Model entries
/// into the two draw-time categories the renderer cares about: batched
/// opaque/mask draws (one instanced GPU call per `(shader, mesh)` pair)
/// and individually-ordered blend draws.
#[derive(Default)]
pub(in crate::core::renderer) struct PassDraws {
    /// Per-(shader_ptr, mesh_ptr) instance buffer + instance count. Keys
    /// match the (shader, mesh) identity a Model attach dedupes on. Bound at vertex slot 1
    /// for the corresponding shader+mesh draw.
    opaque_overrides: HashMap<(usize, usize), (wgpu::Buffer, u32)>,
    /// Translucent draws, **sorted back-to-front** by eye-space Z. The
    /// renderer walks this list inside the blend phase, looking each
    /// shader's pipeline + bind groups up by `shader_ptr`.
    pub(super) blend_draws: Vec<BlendDraw>,
    /// What each shader of the pass draws, in the pass's order.
    pub(in crate::core::renderer) shaders: Vec<ShaderDraw>,
    /// Whether the pass holds Models, so its other shaders ignore the depth attachment.
    pub(super) has_models: bool,
    /// The color and depth targets the pass set.
    pub(in crate::core::renderer) targets: PassTargets,
}

impl PassDraws {
    /// The `(shader, mesh)` pairs a visible Model of the pass draws, opaque or translucent.
    fn collect_drawn(&self) -> HashSet<(usize, usize)> {
        let opaque = self.opaque_overrides.keys().copied();
        let blend = self
            .blend_draws
            .iter()
            .map(|draw| (draw.shader_ptr, Arc::as_ptr(&draw.mesh) as usize));
        opaque.chain(blend).collect()
    }
}

/// What one shader of a render pass draws, decided before the frame is acquired.
pub(in crate::core::renderer) struct ShaderDraw {
    pub(in crate::core::renderer) shader: Arc<ShaderObject>,
    /// The vertex layouts its meshes read, in the order they first appear, one pipeline
    /// each. Empty for a shader with no mesh, which draws with one pipeline that reads no vertex buffer.
    pub(super) layouts: Vec<VertexLayouts>,
    /// Whether the shader draws the pass's Models, hidden ones included.
    pub(super) model: bool,
    pub(in crate::core::renderer) draws: MeshDraws,
}

/// The draws one shader makes in its own turn of a render pass.
pub(in crate::core::renderer) enum MeshDraws {
    /// Three built-in vertices, for a shader with no mesh.
    Fullscreen,
    /// One draw for each mesh of an opaque or masked shader that has something to draw.
    Batched(Vec<MeshDraw>),
    /// None in its own turn: a translucent shader's Models draw in the pass's
    /// blend phase, from the meshes the plan uploaded, by mesh pointer.
    Blend(HashMap<usize, BlendMesh>),
    /// None, with no pipeline: every mesh belongs to Models this frame doesn't draw.
    Skipped,
}

/// One mesh draw: the mesh's buffers, its index count, the instances it draws,
/// and the entry of its shader's `layouts` it reads.
pub(in crate::core::renderer) struct MeshDraw {
    pub(super) buffers: VertexBuffers,
    pub(super) index_count: u32,
    pub(in crate::core::renderer) instances: Instances,
    pub(super) layout: usize,
}

/// A translucent mesh as the plan uploaded and checked it: its buffers, its index
/// count and the entry of its shader's `layouts` it reads. Each of its Models draws
/// these in the blend phase, so a change after the plan waits for the next render.
pub(in crate::core::renderer) struct BlendMesh {
    pub(super) buffers: VertexBuffers,
    pub(super) index_count: u32,
    pub(super) layout: usize,
}

impl RenderContext {
    /// Checks a compute pass before any frame is acquired: each shader against the
    /// device and its textures (`check_shader`, `check_textures`), then its dispatch and
    /// texture accesses (`check_compute`).
    pub(super) fn plan_compute(&self, pass: &PassObject) -> Result<(), RendererError> {
        for shader in pass.shaders.read().iter() {
            shader.flush_pending();
            self.check_shader(shader)?;
            self.check_textures(shader)?;
        }
        self.check_compute(pass)
    }

    /// Decides what a render pass draws, before any frame is acquired.
    ///
    /// The checks that need no frame run here. The target keeps its last frame
    /// when a lit shader holds more lights than the device takes, a shader reads
    /// vertex inputs with no mesh, a mesh is asked for more instances than it holds,
    /// a mesh the device's limits or its own checks refuse (`plan_shader`), a shader
    /// asks for more than the device has (`check_shader`), a texture uniform has no
    /// texture or one its slot can't take (`check_textures`), a viewport side passes
    /// the device's 2D limit (`check_viewport`), or a target the pass set is missing.
    ///
    /// A pass with a target of its own checks its attachments here too.
    /// The draw loop takes each shader's layouts and buffers from the plan.
    /// A pass that draws into the frame checks its attachments once it has one.
    pub(in crate::core::renderer) fn plan_pass(
        &self,
        pass: &PassObject,
    ) -> Result<PassDraws, RendererError> {
        // A mesh's own pass builds its quick view here too, whatever reached the pass.
        pass.ensure_preview();
        self.check_lights(pass)?;
        self.check_viewport(pass)?;
        let targets = self.find_targets(pass)?;
        let mut draws = self.build_pass_draws(pass)?;
        let model_shaders = find_model_shaders(pass);
        draws.has_models = !model_shaders.is_empty();
        let drawn = draws.collect_drawn();
        let shaders = pass.shaders.read().clone();
        for shader in shaders {
            let model = model_shaders.contains(&(Arc::as_ptr(&shader) as usize));
            let plan = self.plan_shader(shader, model, &draws.opaque_overrides, &drawn)?;
            draws.shaders.push(plan);
        }
        draws.targets = targets;
        if let Some(color) = draws.targets.color.as_deref() {
            self.check_attachment(pass, &draws, &Attachment::from_target(color))?;
        }
        Ok(draws)
    }

    /// Decides what `shader` draws in its turn of a pass, given the pass's Model `overrides`.
    /// `model` is true for a shader that draws the pass's Models, hidden ones included.
    /// A Model mesh that no visible Model of the pass draws through this shader, a key
    /// `drawn` lacks, is skipped before it packs, so its checks and limits don't apply.
    fn plan_shader(
        &self,
        shader: Arc<ShaderObject>,
        model: bool,
        overrides: &HashMap<(usize, usize), (wgpu::Buffer, u32)>,
        drawn: &HashSet<(usize, usize)>,
    ) -> Result<ShaderDraw, RendererError> {
        shader.flush_pending();
        self.check_shader(&shader)?;
        self.check_textures(&shader)?;
        // The quick view checks its mesh on every render, however the render
        // reached it, and a mesh it refuses says why on every render.
        let (meshes, passed) = shader.read_meshes()?;
        if meshes.is_empty() {
            require_mesh(&shader)?;
            return Ok(ShaderDraw {
                shader,
                layouts: Vec::new(),
                model,
                draws: MeshDraws::Fullscreen,
            });
        }
        let blend = *shader.alpha_mode.read() == AlphaMode::Blend;
        let shader_ptr = Arc::as_ptr(&shader) as usize;
        let limits = &self.limits;
        let mut layouts: Vec<VertexLayouts> = Vec::new();
        let mut blended = HashMap::new();
        let mut batched = Vec::with_capacity(meshes.len());
        for mesh in meshes.iter() {
            let mesh_ptr = Arc::as_ptr(mesh) as usize;
            if model && !drawn.contains(&(shader_ptr, mesh_ptr)) && mesh.awaits_models() {
                continue;
            }
            // The layouts follow the schemas of the pack the buffers came from.
            let Upload {
                buffers,
                counts,
                schemas,
                ..
            } = match &passed {
                Some(kept) if Arc::ptr_eq(&kept.mesh, mesh) => {
                    self.upload_checked(&shader, mesh, kept.generation)?
                }
                _ => mesh.upload(self)?,
            };
            #[cfg(test)]
            crate::testing::race_hooks::run("uploaded");
            let read = mesh.fetch_layouts(shader.id, &schemas, || {
                vertex_buffer_layouts(&shader, mesh, &schemas)
            })?;
            check_strides(&read, limits)?;
            let layout = match layouts.iter().position(|known| *known == read) {
                Some(index) => index,
                None => {
                    layouts.push(read);
                    layouts.len() - 1
                }
            };
            // A translucent shader's Models draw in the pass's blend phase, from these buffers.
            if blend {
                let index_count = counts.index_count;
                blended.insert(
                    mesh_ptr,
                    BlendMesh {
                        buffers,
                        index_count,
                        layout,
                    },
                );
                continue;
            }
            let models = overrides.get(&(shader_ptr, mesh_ptr));
            let own = buffers.instance_buffer.as_ref();
            let reads = layouts.get(layout);
            let instances = pick_instances(&shader, models, own, mesh, reads, counts)?;
            if let Some(instances) = instances {
                batched.push(MeshDraw {
                    index_count: counts.index_count,
                    instances,
                    buffers,
                    layout,
                });
            }
        }
        let draws = match (layouts.is_empty(), blend) {
            (true, _) => MeshDraws::Skipped,
            (false, true) => MeshDraws::Blend(blended),
            (false, false) => MeshDraws::Batched(batched),
        };
        Ok(ShaderDraw {
            shader,
            layouts,
            model,
            draws,
        })
    }

    /// Per-Model draw buffers gathered from a Pass, split by alpha mode.
    ///
    /// Opaque/Mask entries collapse to one batched instance buffer per
    /// `(shader_ptr, mesh_ptr)` pair — the auto-instancing path. The
    /// renderer binds the buffer at vertex_buffer(1) and issues a single
    /// `draw_indexed(.., 0..count)`, which is the cheapest path for
    /// crowds of shared-material geometry.
    ///
    /// Blend entries are *not* batched: each blend draw needs to be
    /// submitted back-to-front by eye-space depth for correct
    /// over-blending, so we carry per-entry singleton buffers plus a
    /// snapshot of the camera-space Z each entry's transform lands at.
    /// The renderer sorts these by `eye_z` (descending = far→near)
    /// inside the blend phase. If the pass has no camera attached (no
    /// `pass.add(&camera)` call), `eye_z` defaults to 0.0 and the
    /// resulting order is insertion-order — correct for a single
    /// translucent object, best-effort for multiple.
    fn build_pass_draws(&self, pass: &PassObject) -> Result<PassDraws, RendererError> {
        let entries = pass.model_entries.read();
        let mut out = PassDraws::default();
        if entries.is_empty() {
            return Ok(out);
        }

        // Compute eye-Z via the cached view matrix; `None` means no Camera
        // was attached to the pass, in which case we leave `eye_z` at 0
        // and the sort is a no-op (stable, insertion order preserved).
        let view_mat: Option<glam::Mat4> = pass
            .camera_snapshot
            .read()
            .as_ref()
            .map(|snap| glam::Mat4::from_cols_array_2d(&snap.view));

        // Opaque grouping in stable insertion order: a glTF scene that
        // depends on `pass.add(...)` ordering for layering (e.g. additive
        // glow stacked behind a base mesh) was previously seeing random
        // frame-to-frame ordering because `HashMap` iteration is
        // non-deterministic. The linear-scan over a `Vec` is fine for
        // typical pass sizes — most passes have a single-digit number of
        // `(shader, mesh)` groups. The Vec lives on `self.scratch` so the
        // allocation amortizes across frames.
        let mut scratch = self.scratch.write();
        let opaque_groups = &mut scratch.opaque_groups;
        for (_, bytes) in opaque_groups.iter_mut() {
            bytes.clear();
        }
        opaque_groups.clear();
        for entry in entries.iter() {
            // One state read serves both checks. Hidden Models exit before
            // the alpha-mode branch packs an instance row or queues a
            // BlendDraw, so visibility and the transform snapshot stay
            // under a single lock acquisition per entry.
            let model_state = *entry.state.read();
            if !model_state.visible {
                continue;
            }
            let alpha_mode = *entry.shader.alpha_mode.read();
            let key = (
                Arc::as_ptr(&entry.shader) as usize,
                Arc::as_ptr(&entry.mesh) as usize,
            );
            let transform = model_state.transform;
            match alpha_mode {
                AlphaMode::Opaque | AlphaMode::Mask => {
                    let m = transform.to_cols_array_2d();
                    let idx = match opaque_groups.iter().position(|(k, _)| *k == key) {
                        Some(i) => i,
                        None => {
                            opaque_groups.push((key, Vec::new()));
                            opaque_groups.len() - 1
                        }
                    };
                    let slot = &mut opaque_groups[idx].1;
                    slot.extend_from_slice(bytemuck::cast_slice(&m));
                }
                AlphaMode::Blend => {
                    // Eye-space Z of the mesh's local-space AABB centroid in
                    // world coordinates: `view * transform * vec4(centroid, 1)`.
                    // An elongated translucent mesh centered away from its
                    // local origin sorts against its visible extent rather
                    // than its pivot. Empty meshes (sentinel `+inf` / `-inf`)
                    // fall back to the origin so we never multiply a non-finite
                    // value into the view matrix.
                    let (lo, hi) = entry.mesh.aabb_local();
                    let centroid = if lo.is_finite() && hi.is_finite() {
                        0.5 * (lo + hi)
                    } else {
                        glam::Vec3::ZERO
                    };
                    let centroid_world =
                        transform * glam::Vec4::new(centroid.x, centroid.y, centroid.z, 1.0);
                    let eye_z = view_mat.map(|v| (v * centroid_world).z).unwrap_or(0.0);
                    out.blend_draws.push(BlendDraw {
                        shader_ptr: key.0,
                        mesh: entry.mesh.clone(),
                        transform,
                        eye_z,
                        instance_buffer: None,
                    });
                }
            }
        }

        let limit = self.limits.max_buffer_size;
        for (key, bytes) in opaque_groups.iter() {
            check_limit(
                "the pass's Model instance buffer",
                "max_buffer_size",
                bytes.len() as u64,
                limit,
            )?;
            let count = (bytes.len() / 64) as u32;
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pass model instance buffer"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&buffer, 0, bytes);
            out.opaque_overrides.insert(*key, (buffer, count));
        }
        drop(scratch);

        // Back-to-front: highest `eye_z` is closest to the eye in
        // right-handed view space (the camera looks down its -Z axis, so
        // farther geometry has a *more negative* Z and a *smaller* `eye_z`
        // value). We want to draw the smallest values first.
        out.blend_draws.sort_by(|a, b| {
            a.eye_z
                .partial_cmp(&b.eye_z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Materialize one tiny instance buffer per blend draw — 64 bytes
        // each. The per-draw payload is the column-major model matrix
        // already in `transform`, copied via a single bytemuck slice.
        for draw in out.blend_draws.iter_mut() {
            let m = draw.transform.to_cols_array_2d();
            let bytes: &[u8] = bytemuck::cast_slice(&m);
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pass blend model instance buffer"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&buffer, 0, bytes);
            draw.instance_buffer = Some(buffer);
        }

        Ok(out)
    }
}

/// The instances a mesh draws, and the buffer slot 1 steps through for them.
pub(in crate::core::renderer) struct Instances {
    pub(in crate::core::renderer) buffer: Option<wgpu::Buffer>,
    pub(in crate::core::renderer) count: u32,
}

/// The instances one mesh of an opaque or masked shader draws, or `None` when it draws nothing.
///
/// The pass's Models win over the mesh's own instance buffer: `models` holds one matrix
/// for each visible Model that draws this mesh through this shader. Otherwise the mesh
/// draws its own instances, such as a crowd built with `Mesh::add_instance`. A Model
/// mesh that no visible Model draws never gets here: `plan_shader` skips it.
fn pick_instances(
    shader: &ShaderObject,
    models: Option<&(wgpu::Buffer, u32)>,
    own: Option<&wgpu::Buffer>,
    mesh: &MeshObject,
    layouts: Option<&VertexLayouts>,
    counts: DrawCounts,
) -> Result<Option<Instances>, RendererError> {
    // An empty index list draws nothing, like an instance count of 0.
    if counts.index_count == 0 {
        return Ok(None);
    }
    match (models, own) {
        (Some((buffer, count)), _) => Ok(Some(Instances {
            buffer: Some(buffer.clone()),
            count: *count,
        })),
        (None, buffer) => {
            check_instance_count(shader, layouts, counts, mesh)?;
            Ok(Some(Instances {
                buffer: buffer.cloned(),
                count: counts.instance_count,
            }))
        }
    }
}

/// Refuses to draw more instances than the mesh holds
/// when the pipeline reads instance attributes from its buffer.
/// A Model's mesh holds none, and only its Models fill the columns.
fn check_instance_count(
    shader: &ShaderObject,
    layouts: Option<&VertexLayouts>,
    counts: DrawCounts,
    mesh: &MeshObject,
) -> Result<(), RendererError> {
    let rows = counts.instance_rows.unwrap_or(0);
    let Some(instance) = layouts.and_then(|layouts| layouts.instance.as_ref()) else {
        return Ok(());
    };
    if counts.instance_count <= rows {
        return Ok(());
    }
    let message = if mesh.awaits_models() {
        format!(
            "the shader reads the Model columns {}, and no Model in this pass draws this mesh with this shader",
            name_columns(shader, instance)?
        )
    } else {
        format!(
            "instance count {} exceeds the {rows} instances the mesh holds",
            counts.instance_count
        )
    };
    Err(MeshError::InvalidInstance(message).into())
}

/// Names the inputs `shader` reads from the instance stream, in location order:
/// `'model_0', 'model_1', 'model_2' and 'model_3'`.
fn name_columns(shader: &ShaderObject, instance: &VertexStream) -> Result<String, ShaderError> {
    let mut names: Vec<String> = shader
        .reflect_vertex_inputs()?
        .into_iter()
        .filter(|input| {
            instance
                .attributes
                .iter()
                .any(|attribute| attribute.shader_location == input.location)
        })
        .map(|input| format!("'{}'", input.name))
        .collect();
    let Some(last) = names.pop() else {
        return Ok(String::new());
    };
    if names.is_empty() {
        return Ok(last);
    }
    Ok(format!("{} and {last}", names.join(", ")))
}

/// The shaders that draw the pass's Models, hidden ones included.
fn find_model_shaders(pass: &PassObject) -> Vec<usize> {
    let entries = pass.model_entries.read();
    entries
        .iter()
        .map(|entry| Arc::as_ptr(&entry.shader) as usize)
        .collect()
}

/// Refuses a shader with no mesh when it reads vertex inputs, naming the first one.
fn require_mesh(shader: &ShaderObject) -> Result<(), ShaderError> {
    let Some(input) = shader.reflect_vertex_inputs()?.into_iter().next() else {
        return Ok(());
    };
    Err(ShaderError::InvalidKey(format!(
        "shader input '{}' @location({}) has no mesh to read from; attach one with add_mesh",
        input.name, input.location
    )))
}

/// How many times a render checks and uploads a quick view's mesh that keeps changing.
pub(in crate::core::renderer) const PREVIEW_TRIES: usize = 4;
