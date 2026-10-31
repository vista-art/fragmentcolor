//! Records render and compute passes into the command encoder.

use super::bind_groups::{PbrDefaults, report_missing};
use super::draws::{MeshDraws, PassDraws};
use super::samplers::SamplerSources;
use crate::core::PassInput;
use crate::core::renderer::checks::pass::Attachment;
use crate::core::renderer::pipelines::layout::plan_layout;
use crate::core::renderer::pipelines::vertex_layouts::VertexLayouts;
use crate::core::renderer::pipelines::{ComputePipelineKey, PushMode, RenderPipelineKey};
use crate::core::renderer::pipelines::{create_compute_pipeline, create_render_pipeline};
use crate::core::renderer::{Lease, RenderContext, RendererError, TextureKey, pools};
use crate::{PassObject, ScreenRegion, ShaderError, StorageEntry, TargetFrame, UniformData};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// A shader's pipelines and bind groups in one render pass. The blend phase binds them
/// again for each translucent draw. The pipelines share their bind group layouts.
struct ShaderState<'plan> {
    /// One pipeline for each entry of the plan's `layouts`, or one for a shader with no mesh.
    pipelines: Vec<wgpu::RenderPipeline>,
    /// What the plan has the shader draw, the blend phase's meshes included.
    draws: &'plan MeshDraws,
    bind_groups: Vec<(u32, wgpu::BindGroup)>,
    immediates: Option<Vec<u8>>,
}

impl ShaderState<'_> {
    /// Sets the bind groups, the pipeline of `layout` and the native immediates.
    fn bind(&self, render_pass: &mut wgpu::RenderPass<'_>, layout: usize) {
        for (group, bind_group) in self.bind_groups.iter() {
            render_pass.set_bind_group(*group, bind_group, &[]);
        }
        self.switch(render_pass, layout);
    }

    /// Sets the pipeline of `layout` and the native immediates. The bind groups stay
    /// bound, since the shader's pipelines share their layouts, and a pipeline of
    /// another layout clears the immediates.
    fn switch(&self, render_pass: &mut wgpu::RenderPass<'_>, layout: usize) {
        if let Some(pipeline) = self.pipelines.get(layout) {
            render_pass.set_pipeline(pipeline);
        }
        if let Some(bytes) = self.immediates.as_deref() {
            render_pass.set_immediates(0, bytes);
        }
    }
}

impl RenderContext {
    pub(in crate::core::renderer) fn process_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
        frame: &dyn TargetFrame,
        pass_draws: &PassDraws,
    ) -> Result<(), RendererError> {
        // The frame's own size, which a resize on another thread can't change mid-render.
        let size = frame.size();
        // `pass_draws` holds this pass's plan and its per-Model GPU instance
        // buffers, built before the frame was acquired so they outlive the render pass.
        let blend_draws = &pass_draws.blend_draws;

        // Choose color attachment: per-pass override or default frame view.
        // Offscreen color targets are bound single-sampled; surface targets may use MSAA.
        let mut sample_count = self.sample_count();
        let mut resolve_target: Option<&wgpu::TextureView> = None;

        let mut color_format = frame.format();
        let mut extent = size;
        let mut pass_texture_view = None;
        // The plan checked the pass against a color target it set, and the frame is checked here.
        match pass_draws.targets.color.as_deref() {
            Some(texture) => {
                pass_texture_view = Some(texture.create_view());
                color_format = texture.format();
                sample_count = texture.inner.sample_count();
                extent = texture.inner.size();
            }
            None => {
                let attachment = Attachment::from_frame(frame, sample_count);
                self.check_attachment(pass, pass_draws, &attachment)?;
            }
        }

        // Light buffers for every lit shader, binned before the render pass begins.
        let light_buffers = self.prepare_lights(encoder, pass)?;

        let load_op = match *pass.input.read() {
            PassInput::Load => wgpu::LoadOp::Load,
            PassInput::Clear(color) => wgpu::LoadOp::Clear(color.to_wgpu()),
        };

        let viewport = *pass.viewport.read();

        let mut texture_view: &wgpu::TextureView =
            if let Some(pass_view) = pass_texture_view.as_ref() {
                pass_view
            } else {
                frame.view()
            };

        // With MSAA the pass draws into a pooled multisampled texture and resolves
        // into the frame. The lease outlives the render pass and returns it on drop.
        let msaa = (pass_texture_view.is_none() && sample_count > 1).then(|| {
            let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
            let key = TextureKey::new(size, color_format, sample_count, usage);
            Lease::take(&self.texture_pool, &self.device, key)
        });
        if let Some(lease) = msaa.as_ref() {
            texture_view = lease.view();
            resolve_target = Some(frame.view());
        }

        let color_attachments = &[Some(wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })];

        // The depth target the pass set, a pooled one for a pass that draws
        // models, or none. Like the MSAA lease, it outlives the render pass.
        let depth_target = pass_draws.targets.depth.as_deref();
        let depth = self.resolve_depth(pass, depth_target, extent, sample_count);

        let descriptor = wgpu::RenderPassDescriptor {
            label: Some(&format!("Render Pass: {}", pass.name.clone())),
            color_attachments,
            depth_stencil_attachment: depth.describe_attachment(),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        if let Some(region) = viewport {
            // The scissor crops to the attachment, and a region that misses it
            // leaves the pass its clear and no draws.
            let scissor = region.clip(extent.width, extent.height);
            if scissor.width() == 0 || scissor.height() == 0 {
                return Ok(());
            }
            let (x, y, width, height) = viewport_rect(&region);
            render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
            render_pass.set_scissor_rect(
                scissor.min_x,
                scissor.min_y,
                scissor.width(),
                scissor.height(),
            );
        }

        let required_size = *pass.required_buffer_size.read();
        self.buffer_pool
            .write()
            .ensure_capacity(required_size, &self.device);

        // Per-shader render state captured during the opaque phase and
        // looked up again in the global blend phase so translucent draws
        // can switch pipelines/bind-groups as they walk `blend_draws` in
        // back-to-front order across every Material on the pass.
        let mut shader_states: HashMap<usize, ShaderState> = HashMap::new();

        for plan in pass_draws.shaders.iter() {
            // A shader whose meshes all belong to Models this frame skips builds no pipeline.
            if matches!(plan.draws, MeshDraws::Skipped) {
                continue;
            }
            let shader = &plan.shader;
            shader.flush_pending();

            // Snapshot the Material-owned pipeline-state flags from the
            // shader's back-references. Drop the read locks immediately so
            // we don't hold them across the pipeline build.
            let alpha_mode = *shader.alpha_mode.read();
            let double_sided = *shader.double_sided.read();
            // In a pass with models, the shaders that draw none ignore the depth attachment.
            let depth_use = depth.pick(pass_draws.has_models && !plan.model);
            // One pipeline for each vertex layout the shader's meshes read,
            // or one that reads no vertex buffer for a shader with no mesh.
            let keys: Vec<Option<&VertexLayouts>> = match plan.layouts.is_empty() {
                true => vec![None],
                false => plan.layouts.iter().map(Some).collect(),
            };
            let mut pipelines = Vec::with_capacity(keys.len());
            for layouts in keys {
                let pipeline_key = RenderPipelineKey {
                    shader_hash: shader.hash,
                    color_format,
                    depth: depth_use,
                    sample_count,
                    alpha_mode,
                    double_sided,
                    vertex_layouts: layouts.cloned(),
                };
                // The pipeline leaves the cache guard here, so the next lookup takes no other guard.
                let pipeline = self
                    .fetch_pipeline(
                        &self.render_pipelines,
                        pipeline_key,
                        "render pipeline",
                        || {
                            let layout = plan_layout(shader, self, Some(self.light_path()));
                            create_render_pipeline(
                                &self.device,
                                shader,
                                &layout,
                                color_format,
                                sample_count,
                                layouts,
                                depth_use,
                                alpha_mode,
                                double_sided,
                                self.aligned_uniform_bindings.load(Ordering::Relaxed),
                                self.light_path(),
                                self.writes_wgsl.load(Ordering::Relaxed),
                            )
                        },
                    )?
                    .value()
                    .clone();
                pipelines.push(pipeline);
            }
            // Every pipeline of a shader has the same bind group layouts, and wgpu
            // dedupes equal layouts, so one set of bind groups serves them all.
            let Some(cached_pipeline) = pipelines.first() else {
                continue;
            };

            // Collect resources per bind group to build entries safely with owned views/samplers
            #[derive(Default)]
            struct BindGroupResources {
                uniform_buffers: Vec<(u32, pools::BufferLocation)>,
                // legacy storage-pool backed buffers (kept for compatibility in other paths)
                storage_buffers: Vec<(u32, pools::BufferLocation)>,
                // persistent storage buffers (buffer, span)
                persistent_storage_buffers: Vec<(u32, wgpu::Buffer, u64)>,
                // light set buffers, bound whole
                light_buffers: Vec<(u32, wgpu::Buffer)>,
                sampler_sources: SamplerSources,
                views: Vec<(u32, wgpu::TextureView)>,
            }

            let mut groups: HashMap<u32, BindGroupResources> = HashMap::new();
            if let Some(roots) = shader.light_roots.as_ref() {
                let buffers = light_buffers
                    .get(&(Arc::as_ptr(shader) as usize))
                    .ok_or_else(|| ShaderError::UniformNotFound("lights".into()))?;
                for (group, binding, buffer) in buffers.list_bindings(roots) {
                    groups
                        .entry(group)
                        .or_default()
                        .light_buffers
                        .push((binding, buffer));
                }
            }
            let mut pbr_defaults_handle: Option<Arc<PbrDefaults>> = None;
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                match &uniform.data {
                    UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = self.validate_step(
                                &format!("texture view for binding '{}'", name),
                                || Ok(tex.create_view()),
                            )?;
                            let group_entry = groups.entry(uniform.group).or_default();
                            group_entry.views.push((uniform.binding, view));
                            if meta.sampled {
                                group_entry
                                    .sampler_sources
                                    .add_texture(uniform.binding, tex);
                            }
                        } else {
                            // No user texture — fall back to the PBR default for
                            // the known glTF slot names so `Material::pbr` works
                            // without the caller supplying every map. The 1x1
                            // defaults look the same under any sampler, so they
                            // don't pair with one.
                            let defaults =
                                pbr_defaults_handle.get_or_insert_with(|| self.pbr_defaults());
                            let Some(view) = defaults.slot(name) else {
                                return Err(report_missing(name, meta));
                            };
                            groups
                                .entry(uniform.group)
                                .or_default()
                                .views
                                .push((uniform.binding, view.clone()));
                        }
                    }
                    UniformData::Sampler(info) => {
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .sampler_sources
                            .add_sampler(uniform.binding, info.comparison);
                    }
                    UniformData::Storage(data) => {
                        if let Some(StorageEntry { span, .. }) = data.first() {
                            let span_u64 = *span as u64;
                            let buf = self.storage_buffer(shader, name, span_u64);

                            groups
                                .entry(uniform.group)
                                .or_default()
                                .persistent_storage_buffers
                                .push((uniform.binding, buf, span_u64));
                        } else {
                            return Err(ShaderError::UniformNotFound(name.clone()).into());
                        }
                    }
                    UniformData::PushConstant(_) => {
                        if let Some((group, binding, location)) =
                            self.upload_push(shader, name, cached_pipeline.push_mode.as_ref())?
                        {
                            groups
                                .entry(group)
                                .or_default()
                                .uniform_buffers
                                .push((binding, location));
                        }
                    }
                    _ => {
                        let storage = shader.storage.read();
                        let bytes = storage
                            .get_bytes(name)
                            .ok_or(ShaderError::UniformNotFound(name.clone()))?;

                        let buffer_location = {
                            let mut buffer_pool = self.buffer_pool.write();
                            buffer_pool.upload(bytes, &self.queue, &self.device)
                        };

                        groups
                            .entry(uniform.group)
                            .or_default()
                            .uniform_buffers
                            .push((uniform.binding, buffer_location));
                    }
                }
            }

            // Build bind groups per layout (by group index)
            let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
            let mut present_groups: HashSet<u32> = HashSet::new();

            for (group_index, resources) in groups.into_iter() {
                let Some(layout) = cached_pipeline.bind_group_layouts.get(&group_index) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group_index
                    )));
                };

                // Assemble entries borrowing from owned resources and buffer pool
                let samplers = resources.sampler_sources.resolve(&self.device);
                let buffer_pool = self.buffer_pool.read();
                let storage_pool = self.storage_pool.read();
                let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                for (binding, loc) in resources.uniform_buffers.iter() {
                    let binding_ref = buffer_pool.get_binding(*loc);
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(binding_ref),
                    });
                }
                for (binding, loc) in resources.storage_buffers.iter() {
                    let binding_ref = storage_pool.get_binding(*loc);
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(binding_ref),
                    });
                }
                // Persistent buffers
                for (binding, buf, span) in resources.persistent_storage_buffers.iter() {
                    // Bind exactly the reflected span; do not pad, to avoid range overflow
                    let size_nz = unsafe { std::num::NonZeroU64::new_unchecked(*span) };
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buf,
                            offset: 0,
                            size: Some(size_nz),
                        }),
                    });
                }
                for (binding, buffer) in resources.light_buffers.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: buffer.as_entire_binding(),
                    });
                }
                for (binding, view) in resources.views.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }
                for (binding, sampler) in samplers.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    });
                }

                // Sort by binding index to match layout order
                entries.sort_by_key(|e| e.binding);

                let label = format!("Bind Group for group: {}", group_index);
                let bind_group = self.validate_step(&label, || {
                    Ok(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &entries,
                        label: Some(&label),
                    }))
                })?;
                present_groups.insert(group_index);
                bind_groups.push((group_index, bind_group));
            }

            let empty =
                self.create_empty_groups(&cached_pipeline.bind_group_layouts, &present_groups)?;
            bind_groups.extend(empty);

            // Sort by group index to match pipeline layout order
            bind_groups.sort_by_key(|(g, _)| *g);

            // Native immediate data, set right after each pipeline.
            // Blocking read for the same reason as the storage-buffer snapshot
            // earlier in this function.
            let mut immediate_bytes: Option<Vec<u8>> = None;
            if let Some(PushMode::Native { root, .. }) = &cached_pipeline.push_mode {
                let storage = shader.storage.read();
                immediate_bytes = storage.get_bytes(root).map(<[u8]>::to_vec);
            }

            // Snapshot the pipelines + bind-groups + native immediate bytes
            // so the draws below and the blend phase can bind them again without
            // walking the bind-group construction again. wgpu types are
            // Arc-backed, so the clones are cheap (single refcount bump).
            let state = ShaderState {
                pipelines: pipelines
                    .into_iter()
                    .map(|cached| cached.pipeline)
                    .collect(),
                draws: &plan.draws,
                bind_groups,
                immediates: immediate_bytes,
            };
            state.bind(&mut render_pass, 0);

            match &plan.draws {
                MeshDraws::Fullscreen => render_pass.draw(0..3, 0..1),
                // Auto-instancing path: one batched draw per (shader, mesh) pair
                // regardless of how many Models contributed. The pass packs all
                // queued model matrices into one vertex buffer at slot 1.
                // A mesh with another layout than the one before switches pipelines.
                MeshDraws::Batched(meshes) => {
                    let mut current = 0;
                    for mesh in meshes {
                        if mesh.layout != current {
                            state.switch(&mut render_pass, mesh.layout);
                            current = mesh.layout;
                        }
                        if let Some(buffer) = mesh.instances.buffer.as_ref() {
                            render_pass.set_vertex_buffer(1, buffer.slice(..));
                        }
                        render_pass.set_vertex_buffer(0, mesh.buffers.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.buffers.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..mesh.instances.count);
                    }
                }
                // Blend draws run in the global back-to-front phase below, after every
                // shader has registered its pipeline and bind groups in `shader_states`.
                MeshDraws::Blend(_) | MeshDraws::Skipped => {}
            }
            shader_states.insert(Arc::as_ptr(shader) as usize, state);
        }

        // Global blend phase: walk every `BlendDraw` in eye-Z far→near
        // order (already sorted in `build_pass_draws`) and rebind the
        // owning shader's pipeline + bind groups whenever the shader or
        // the mesh's layout changes. Two translucent Materials in one Pass
        // interleave correctly — A-far, B-mid, A-near sequences emit in
        // global Z order rather than per-shader buckets.
        #[cfg(test)]
        crate::testing::race_hooks::run("blend phase");
        if !blend_draws.is_empty() {
            let mut current: Option<(usize, usize)> = None;
            for draw in blend_draws.iter() {
                let Some(buf) = draw.instance_buffer.as_ref() else {
                    continue;
                };
                let Some(state) = shader_states.get(&draw.shader_ptr) else {
                    continue;
                };
                // The buffers the plan uploaded, read with the layout the plan built for them.
                let MeshDraws::Blend(meshes) = state.draws else {
                    continue;
                };
                let Some(mesh) = meshes.get(&(Arc::as_ptr(&draw.mesh) as usize)) else {
                    continue;
                };
                match current {
                    Some((shader, bound)) if shader == draw.shader_ptr && bound == mesh.layout => {}
                    Some((shader, _)) if shader == draw.shader_ptr => {
                        state.switch(&mut render_pass, mesh.layout)
                    }
                    _ => state.bind(&mut render_pass, mesh.layout),
                }
                current = Some((draw.shader_ptr, mesh.layout));
                if mesh.index_count == 0 {
                    continue;
                }
                render_pass.set_vertex_buffer(0, mesh.buffers.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, buf.slice(..));
                render_pass.set_index_buffer(
                    mesh.buffers.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        Ok(())
    }

    pub(super) fn process_compute_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
    ) -> Result<(), RendererError> {
        // Similar resource binding as render pass, but no target
        for (shader_index, shader) in pass.shaders.read().iter().enumerate() {
            shader.flush_pending();
            let layout = plan_layout(shader, self, None);
            let key = ComputePipelineKey {
                shader_hash: shader.hash,
                bind_group_entries: layout.list_entries(),
            };
            let cached =
                self.fetch_pipeline(&self.compute_pipelines, key, "compute pipeline", || {
                    let writes_wgsl = self.writes_wgsl.load(Ordering::Relaxed);
                    let pad = self.aligned_uniform_bindings.load(Ordering::Relaxed);
                    create_compute_pipeline(&self.device, shader, &layout, writes_wgsl, pad)
                })?;
            #[derive(Default)]
            struct BindGroupResources {
                buffers: Vec<(u32, pools::BufferLocation)>,
                storage_buffers: Vec<(u32, wgpu::Buffer, u64)>,
                views: Vec<(u32, wgpu::TextureView)>,
                sampler_sources: SamplerSources,
            }
            let mut groups: HashMap<u32, BindGroupResources> = HashMap::new();
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;
                match &uniform.data {
                    UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = self.validate_step(
                                &format!("texture view for binding '{}'", name),
                                || Ok(tex.create_view()),
                            )?;
                            let group_entry = groups.entry(uniform.group).or_default();
                            group_entry.views.push((uniform.binding, view));
                            if meta.sampled {
                                group_entry
                                    .sampler_sources
                                    .add_texture(uniform.binding, tex);
                            }
                        } else {
                            return Err(report_missing(name, meta));
                        }
                    }
                    UniformData::Sampler(info) => {
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .sampler_sources
                            .add_sampler(uniform.binding, info.comparison);
                    }
                    UniformData::PushConstant(_) => {
                        if let Some((group, binding, location)) =
                            self.upload_push(shader, name, cached.push_mode.as_ref())?
                        {
                            groups
                                .entry(group)
                                .or_default()
                                .buffers
                                .push((binding, location));
                        }
                    }
                    UniformData::Storage(data) => {
                        if let Some(StorageEntry { span: span_u32, .. }) = data.first() {
                            let span = *span_u32 as u64;
                            let buf = self.storage_buffer(shader, name, span);
                            groups
                                .entry(uniform.group)
                                .or_default()
                                .storage_buffers
                                .push((uniform.binding, buf, span));
                        }
                    }
                    _ => {
                        // Blocking read — same rationale as the storage-buffer
                        // path above.
                        let storage = shader.storage.read();
                        let bytes = storage
                            .get_bytes(name)
                            .ok_or(ShaderError::UniformNotFound(name.clone()))?;
                        let loc = {
                            let mut pool = self.buffer_pool.write();
                            pool.upload(bytes, &self.queue, &self.device)
                        };
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .buffers
                            .push((uniform.binding, loc));
                    }
                }
            }

            let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
            let mut present_groups: HashSet<u32> = HashSet::new();
            for (group, owned) in groups.into_iter() {
                let Some(layout) = cached.bind_group_layouts.get(&group) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group
                    )));
                };
                let samplers = owned.sampler_sources.resolve(&self.device);
                let buffer_pool = self.buffer_pool.read();
                let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                for (binding, loc) in owned.buffers.iter() {
                    let binding_ref = buffer_pool.get_binding(*loc);
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(binding_ref),
                    });
                }
                // Persistent buffers
                for (binding, buf, span) in owned.storage_buffers.iter() {
                    // Bind exactly the reflected span; do not pad, to avoid range overflow
                    let size_nz = unsafe { std::num::NonZeroU64::new_unchecked(*span) };
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: buf,
                            offset: 0,
                            size: Some(size_nz),
                        }),
                    });
                }
                for (binding, view) in owned.views.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }
                for (binding, sampler) in samplers.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    });
                }
                entries.sort_by_key(|e| e.binding);
                let label = format!("Compute Bind Group for group: {}", group);
                let bind_group = self.validate_step(&label, || {
                    Ok(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &entries,
                        label: Some(&label),
                    }))
                })?;
                present_groups.insert(group);
                bind_groups.push((group, bind_group));
            }
            let empty = self.create_empty_groups(&cached.bind_group_layouts, &present_groups)?;
            bind_groups.extend(empty);
            bind_groups.sort_by_key(|(group_index, _)| *group_index);

            let mut ccompute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!(
                    "Compute Pass: {} shader: {}",
                    pass.name.clone(),
                    shader_index
                )),
                timestamp_writes: None,
            });

            ccompute_pass.set_pipeline(&cached.pipeline);
            for (index, bind_group) in bind_groups.iter() {
                ccompute_pass.set_bind_group(*index, bind_group, &[]);
            }
            if let Some(PushMode::Native { root, .. }) = &cached.push_mode
                && let Some(bytes) = shader.storage.read().get_bytes(root)
            {
                ccompute_pass.set_immediates(0, bytes);
            }

            let (x, y, z) = pass.compute_dispatch();
            ccompute_pass.dispatch_workgroups(x, y, z);
        }

        Ok(())
    }
}

/// The viewport for `region`, as given. `check_viewport` keeps each side within
/// the device's 2D limit, and a region that starts past the attachment draws nothing,
/// so the far edge stays below twice that limit, the bound wgpu takes.
fn viewport_rect(region: &ScreenRegion) -> (f32, f32, f32, f32) {
    let (x, y) = (region.min_x as f32, region.min_y as f32);
    (x, y, region.width() as f32, region.height() as f32)
}
