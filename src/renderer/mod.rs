use crate::{
    PassObject, ShaderHash, Target, TargetFrame, TextureInput, TextureOptions, TextureTarget,
    shader::Uniform,
};
use crate::{Size, WindowTarget};
use dashmap::DashMap;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
pub type Commands = Vec<wgpu::CommandBuffer>;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::prelude::*;

pub mod platform;
pub use platform::*;

pub mod error;
pub use error::*;

pub mod renderable;
pub use renderable::*;

mod buffer_pool;
pub use buffer_pool::*;

mod texture_pool;
pub use texture_pool::*;

/// The Renderer accepts a generic window handle as input
/// that must report its display size.
pub trait HasDisplaySize {
    fn size(&self) -> Size;
}

#[derive(Debug)]
struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    // Map of bind group index -> layout (keeps group indices stable)
    bind_group_layouts: std::collections::HashMap<u32, wgpu::BindGroupLayout>,
}

#[derive(Debug, Default)]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[lsp_doc("docs/api/core/renderer/renderer.md")]
pub struct Renderer {
    instance: RwLock<Option<Arc<wgpu::Instance>>>,
    adapter: RwLock<Option<wgpu::Adapter>>,
    context: RwLock<Option<Arc<RenderContext>>>,
}

impl Renderer {
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new() -> Self {
        Renderer {
            instance: RwLock::new(None),
            adapter: RwLock::new(None),
            context: RwLock::new(None),
        }
    }

    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target(
        &self,
        window: impl Into<wgpu::SurfaceTarget<'static>> + HasDisplaySize,
    ) -> Result<crate::RenderTarget, InitializationError> {
        let sz = window.size();
        let size = wgpu::Extent3d {
            width: sz.width,
            height: sz.height,
            depth_or_array_layers: 1,
        };

        match self.create_surface(window, size).await {
            Ok((context, surface, config)) => Ok(crate::RenderTarget::from(WindowTarget::new(
                context, surface, config,
            ))),
            Err(InitializationError::SurfaceError(e)) => {
                // Could not create a surface from the provided window handle.
                // Fall back to a texture-backed target so CI/tests and headless
                // environments can still render.
                log::warn!(
                    "create_target: surface creation failed ({}). Falling back to TextureTarget.",
                    e
                );
                let tex = self
                    .create_texture_target([size.width, size.height])
                    .await?;
                Ok(crate::RenderTarget::from(tex))
            }
            Err(e) => Err(e),
        }
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target(
        &self,
        size: impl Into<Size>,
    ) -> Result<TextureTarget, InitializationError> {
        let context = self.context(None).await?;
        if let Some(adapter) = self.adapter.read().as_ref() {
            let sample_count = crate::renderer::platform::all::pick_sample_count(
                adapter,
                4,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            context.set_sample_count(sample_count);
        } else {
            context.set_sample_count(1);
        }
        let texture = TextureTarget::new(context, size.into(), wgpu::TextureFormat::Rgba8Unorm);

        Ok(texture)
    }

    /// Create a Texture from a unified input; returns the public Texture wrapper.
    /// This variant infers size/format from the input when possible (encoded image bytes, file path).
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture(
        &self,
        input: impl Into<crate::texture::TextureInput>,
    ) -> Result<crate::texture::Texture, RendererError> {
        self.create_texture_with(input, crate::texture::TextureOptions::default())
            .await
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_with_size.md")]
    pub async fn create_texture_with_size(
        &self,
        input: impl Into<TextureInput>,
        size: impl Into<Size>,
    ) -> Result<crate::texture::Texture, RendererError> {
        let options = TextureOptions {
            size: Some(size.into()),
            ..Default::default()
        };
        self.create_texture_with(input, options).await
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_with_format.md")]
    pub async fn create_texture_with_format(
        &self,
        input: impl Into<TextureInput>,
        format: impl Into<crate::texture::TextureFormat>,
    ) -> Result<crate::texture::Texture, RendererError> {
        let options = TextureOptions {
            format: format.into(),
            ..Default::default()
        };
        self.create_texture_with(input, options).await
    }

    #[lsp_doc("docs/api/core/renderer/create_texture_with.md")]
    pub async fn create_texture_with(
        &self,
        input: impl Into<TextureInput>,
        options: impl Into<TextureOptions>,
    ) -> Result<crate::texture::Texture, RendererError> {
        let options = options.into();
        let context = self.context(None).await?;
        match input.into() {
            //
            // From Bytes
            TextureInput::Bytes(bytes) => {
                let object = if let (Some(sz), fmt) = (options.size, options.format) {
                    let wfmt: wgpu::TextureFormat = fmt.into();
                    crate::TextureObject::from_raw_bytes(context.as_ref(), sz.into(), wfmt, &bytes)?
                } else {
                    crate::TextureObject::from_bytes(context.as_ref(), &bytes)?
                };
                let object = std::sync::Arc::new(object);
                let id = context.register_texture(object.clone());
                Ok(crate::texture::Texture::new(context, object, id))
            }
            //
            // From Path
            TextureInput::Path(path) => {
                let object = crate::TextureObject::from_file(context.as_ref(), path)
                    .map_err(|e| InitializationError::Error(format!("{}", e)))?;
                let object = std::sync::Arc::new(object);
                let id = context.register_texture(object.clone());

                Ok(crate::texture::Texture::new(context, object, id))
            }
            //
            // From another Texture
            TextureInput::CloneOf(tex) => Ok(tex),
            //
            // From a URL
            TextureInput::Url(url) => {
                let bytes = crate::net::fetch_bytes(&url).await?;

                let object = Arc::new(crate::TextureObject::from_bytes(context.as_ref(), &bytes)?);
                let id = context.register_texture(object.clone());

                Ok(crate::texture::Texture::new(context, object, id))
            }
            //
            // From a DynamicImage
            TextureInput::DynamicImage(dynamic_image) => {
                let object =
                    crate::TextureObject::from_loaded_image(context.as_ref(), &dynamic_image);
                let object = std::sync::Arc::new(object);
                let id = context.register_texture(object.clone());

                Ok(crate::texture::Texture::new(context, object, id))
            }
        }
    }

    /// Create a storage-class texture with optional explicit usage (default: STORAGE|TEXTURE|COPY_SRC|COPY_DST).
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture(
        &self,
        size: impl Into<crate::Size>,
        format: wgpu::TextureFormat,
        usage: Option<wgpu::TextureUsages>,
    ) -> Result<crate::texture::Texture, InitializationError> {
        let context = self.context(None).await?;
        let usage = usage.unwrap_or(
            wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        );
        let obj = crate::TextureObject::new(
            context.as_ref(),
            size.into().into(),
            format,
            usage,
            crate::texture::SamplerOptions::default(),
        );
        let obj = std::sync::Arc::new(obj);
        let id = context.register_texture(obj.clone());
        Ok(crate::texture::Texture::new(context, obj, id))
    }

    /// Create a depth texture (Depth32Float).
    #[lsp_doc("docs/api/core/renderer/create_depth_texture.md")]
    pub async fn create_depth_texture(
        &self,
        size: impl Into<crate::Size>,
    ) -> Result<crate::texture::Texture, InitializationError> {
        let context = self.context(None).await?;
        let obj = crate::TextureObject::create_depth_texture(context.as_ref(), size.into().into());
        let obj = std::sync::Arc::new(obj);
        let id = context.register_texture(obj.clone());
        Ok(crate::texture::Texture::new(context, obj, id))
    }

    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        if let Some(context) = self.context.read().as_ref() {
            context.render(renderable, target)
        } else {
            Err(RendererError::NoContext)
        }
    }

    pub(crate) async fn create_surface<'window>(
        &self,
        handle: impl Into<wgpu::SurfaceTarget<'window>>,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'window>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let instance = self.instance().await;
        let surface = instance.create_surface(handle)?;
        let context = self.context(Some(&surface)).await?;

        let adapter = self.adapter.read();
        let config = configure_surface(&context.device, adapter.as_ref().unwrap(), &surface, &size);

        // Negotiate and store effective sample count (currently default wanted=1; configurable later)
        if let Some(adapter_ref) = adapter.as_ref() {
            let sc = platform::all::pick_sample_count(adapter_ref, 1, config.format);
            context.set_sample_count(sc);
        }

        Ok((context, surface, config))
    }

    async fn instance(&self) -> Arc<wgpu::Instance> {
        if let Some(instance) = self.instance.read().as_ref() {
            instance.clone()
        } else {
            let instance = platform::all::create_instance().await;
            let instance = Arc::new(instance);
            self.instance.write().replace(instance.clone());
            instance
        }
    }

    async fn context(
        &self,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Arc<RenderContext>, InitializationError> {
        let context = if let Some(context) = self.context.read().as_ref() {
            context.clone()
        } else {
            let instance = self.instance().await;
            let adapter = request_adapter(instance.as_ref(), surface).await?;
            let (device, queue) = request_device(&adapter).await?;
            let context = Arc::new(RenderContext::new(device, queue));

            self.adapter.write().replace(adapter);
            self.context.write().replace(context.clone());

            context
        };

        Ok(context)
    }
}

#[doc(hidden)]
#[derive(Debug)]
/// Draws things on the screen or a texture.
///
/// It owns and manages all GPU resources, serving as the
/// main graphics context provider for the application.
pub struct RenderContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    // Cache RenderPipelines by (shader hash, target format, sample count, layout signature)
    // layout_sig = 0 means shader-only (no vertex/instance streams)
    render_pipelines: DashMap<(ShaderHash, wgpu::TextureFormat, u32, u64), RenderPipeline>,
    buffer_pool: RwLock<UniformBufferPool>,
    // Storage buffer pool (STORAGE | COPY_DST), separate from uniform pool
    storage_pool: RwLock<UniformBufferPool>,
    pub(crate) readback_pool: RwLock<ReadbackBufferPool>,
    pub(crate) texture_pool: RwLock<TexturePool>,

    // Texture registry: id -> TextureObject
    textures: DashMap<crate::texture::TextureId, Arc<crate::TextureObject>>,
    next_id: AtomicU64,

    // MSAA sample count negotiated for current target/format
    sample_count: AtomicU32,
}

impl RenderContext {
    /// Creates a new Context with the given device and queue.
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let buffer_pool = UniformBufferPool::new("Uniform Buffer Pool", &device);
        let storage_pool = UniformBufferPool::with_params(
            "Storage Buffer Pool",
            &device,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            0x10000,
        );

        RenderContext {
            device,
            queue,

            render_pipelines: DashMap::new(),
            buffer_pool: RwLock::new(buffer_pool),
            storage_pool: RwLock::new(storage_pool),
            readback_pool: RwLock::new(ReadbackBufferPool::new("Readback Buffer Pool", 8)),
            texture_pool: RwLock::new(TexturePool::new(16)),

            textures: DashMap::new(),
            next_id: AtomicU64::new(1),
            sample_count: AtomicU32::new(1),
        }
    }

    /// Renders a Frame or Shader to a Target.
    fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        log::info!("[render] before get_current_frame");
        let frame = self.try_get_frame_with_retry(target)?;
        log::info!("[render] got current frame");

        for pass in renderable.passes() {
            if pass.is_compute() {
                self.process_compute_pass(&mut encoder, pass)?
            } else {
                self.process_render_pass(&mut encoder, pass, frame.as_ref(), target.size().into())?
            }
        }

        self.queue.submit(Some(encoder.finish()));

        if frame.auto_present() {
            frame.present();
        }

        #[cfg(feature = "fc_metrics")]
        {
            let u = self.buffer_pool.read().stats();
            let r = self.readback_pool.read().stats();
            let t = self.texture_pool.read().stats();
            log::debug!(
                "pool stats: uniform: alloc={} bytes={} | readback: hits={} misses={} evict={} alloc={} bytes={} | texture: hits={} misses={} evict={} alloc={} bytes={}",
                u.allocations,
                u.bytes_allocated,
                r.hits,
                r.misses,
                r.evictions,
                r.allocations,
                r.bytes_allocated,
                t.hits,
                t.misses,
                t.evictions,
                t.allocations,
                t.bytes_allocated
            );
        }

        Ok(())
    }

    /// Try to get a frame once, and on Lost/Outdated, retry exactly once.
    /// This is a centralized, generic helper; specific targets may still
    /// perform their own recovery internally (e.g., WindowTarget).
    fn try_get_frame_with_retry(
        &self,
        target: &impl Target,
    ) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        match target.get_current_frame() {
            Ok(f) => Ok(f),
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                // Retry exactly once.
                target.get_current_frame()
            }
            Err(e) => Err(e),
        }
    }
}

impl RenderContext {
    fn register_texture(&self, tex: Arc<crate::TextureObject>) -> crate::texture::TextureId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let tid = crate::texture::TextureId(id);
        self.textures.insert(tid.clone(), tex);
        tid
    }

    fn get_texture(&self, id: &crate::texture::TextureId) -> Option<Arc<crate::TextureObject>> {
        self.textures.get(id).map(|r| r.clone())
    }

    fn set_sample_count(&self, n: u32) {
        self.sample_count.store(n.max(1), Ordering::Relaxed);
    }

    fn sample_count(&self) -> u32 {
        self.sample_count.load(Ordering::Relaxed).max(1)
    }

    fn process_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
        frame: &dyn TargetFrame,
        size: wgpu::Extent3d,
    ) -> Result<(), RendererError> {
        self.buffer_pool.write().reset();
        self.storage_pool.write().reset();

        let load_op = match pass.get_input().load {
            true => wgpu::LoadOp::Load,
            false => wgpu::LoadOp::Clear(pass.get_input().color.into()),
        };

        // Choose color attachment: direct frame view (single-sample) or MSAA view with resolve
        let sc = self.sample_count();
        let fmt = frame.format();
        let mut view: &wgpu::TextureView = frame.view();
        let mut resolve_target: Option<&wgpu::TextureView> = None;
        // Keep MSAA resources alive for the duration of the pass
        let mut msaa_texture: Option<wgpu::Texture> = None;
        let mut _msaa_view: Option<wgpu::TextureView> = None;

        if sc > 1 {
            let key = TextureKey::new(size, fmt, sc, wgpu::TextureUsages::RENDER_ATTACHMENT);
            let texture = {
                let mut pool = self.texture_pool.write();
                pool.acquire(&self.device, key)
            };
            _msaa_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            msaa_texture = Some(texture);
            view = _msaa_view.as_ref().unwrap();
            resolve_target = Some(frame.view());
        }

        let attachments = &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })];

        let descriptor = wgpu::RenderPassDescriptor {
            label: Some(&format!("Render Pass: {}", pass.name.clone())),
            color_attachments: attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        log::info!(
            "[render] !!!!!!! calling begin_render_pass with {:?}",
            &descriptor
        );
        let mut render_pass = encoder.begin_render_pass(&descriptor);
        log::info!("[render] begin_render_pass OK");

        // Defaults to Color::TRANSPARENT
        // render_pass.set_blend_constant(wgpu::Color::WHITE);

        let required_size = *pass.required_buffer_size.read();
        self.buffer_pool
            .write()
            .ensure_capacity(required_size, &self.device);

        for shader in pass.shaders.read().iter() {
            let format = frame.format();
            let sc = self.sample_count();

            // Determine mesh (if any) for this pass
            let mesh_ref = pass.mesh.read().clone();
            let mut cached_refs: Option<(crate::mesh::GpuOwned, crate::mesh::DrawCounts)> = None;
            let (layout_sig, vertex_buffer_layouts) = if let Some(mo) = mesh_ref.as_ref() {
                let sig = mo.layout_signature();
                // Ensure GPU now to obtain layouts
                let (refs, counts, layouts) = mo
                    .ensure_gpu(&self.device, &self.queue)
                    .map_err(|e| RendererError::Error(e.to_string()))?;
                cached_refs = Some((refs, counts));
                (sig, Some((layouts.vertex, layouts.instance)))
            } else {
                (0u64, None)
            };

            let cached = self
                .render_pipelines
                .entry((shader.hash, format, sc, layout_sig))
                .or_insert_with(|| {
                    let layouts =
                        create_bind_group_layouts(&self.device, &shader.storage.read().uniforms);
                    let pipeline = create_render_pipeline(
                        &self.device,
                        &layouts,
                        shader,
                        format,
                        sc,
                        vertex_buffer_layouts.as_ref().map(|(v, i)| (v, i.as_ref())),
                    );

                    RenderPipeline {
                        pipeline,
                        bind_group_layouts: layouts,
                    }
                });

            // Collect resources per bind group to build entries safely with owned views/samplers
            #[derive(Default)]
            struct GroupOwned {
                buffers: Vec<(u32, buffer_pool::BufferLocation)>,
                views: Vec<(u32, wgpu::TextureView)>,
                samplers: Vec<(u32, wgpu::Sampler)>,
                last_tex_sampler: Option<wgpu::Sampler>,
            }

            let mut groups: HashMap<u32, GroupOwned> = HashMap::new();
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                match &uniform.data {
                    crate::shader::uniform::UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = tex.create_view();
                            let sampler = tex.sampler();
                            let group_entry = groups.entry(uniform.group).or_default();
                            group_entry.views.push((uniform.binding, view));
                            group_entry.last_tex_sampler = Some(sampler);
                        } else {
                            log::warn!(
                                "Texture handle {:?} not found for uniform {}",
                                meta.id,
                                name
                            );
                        }
                    }
                    crate::shader::uniform::UniformData::Sampler(_) => {
                        // If a texture in the same group was seen, use its sampler; otherwise, fallback
                        let default_sampler = self
                            .device
                            .create_sampler(&wgpu::SamplerDescriptor::default());
                        let sampler = groups
                            .entry(uniform.group)
                            .or_default()
                            .last_tex_sampler
                            .clone()
                            .unwrap_or(default_sampler);
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .samplers
                            .push((uniform.binding, sampler));
                    }
                    crate::shader::uniform::UniformData::Storage(data) => {
                        if let Some((_inner, span, _access)) = data.first() {
                            // Upload the current CPU blob for this storage buffer
                            let storage = shader.storage.read();
                            let blob = storage
                                .storage_blobs
                                .get(name)
                                .cloned()
                                .unwrap_or_else(|| vec![0u8; *span as usize]);
                            let buffer_location = {
                                let mut pool = self.storage_pool.write();
                                pool.upload(&blob, &self.queue, &self.device)
                            };
                            groups
                                .entry(uniform.group)
                                .or_default()
                                .buffers
                                .push((uniform.binding, buffer_location));
                        } else {
                            return Err(crate::ShaderError::UniformNotFound(name.clone()).into());
                        }
                    }
                    _ => {
                        let storage = shader.storage.read();
                        let bytes = storage
                            .get_bytes(name)
                            .ok_or(crate::ShaderError::UniformNotFound(name.clone()))?;

                        let buffer_location = {
                            let mut buffer_pool = self.buffer_pool.write();
                            buffer_pool.upload(bytes, &self.queue, &self.device)
                        };

                        groups
                            .entry(uniform.group)
                            .or_default()
                            .buffers
                            .push((uniform.binding, buffer_location));
                    }
                }
            }

            // Build bind groups per layout (by group index)
            let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
            for (group, owned) in groups.into_iter() {
                let Some(layout) = cached.bind_group_layouts.get(&group) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group
                    )));
                };

                // Assemble entries borrowing from owned resources and buffer pool
                let buffer_pool = self.buffer_pool.read();
                let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                for (binding, loc) in owned.buffers.iter() {
                    let binding_ref = buffer_pool.get_binding(*loc);
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(binding_ref),
                    });
                }
                for (binding, view) in owned.views.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }
                for (binding, sampler) in owned.samplers.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    });
                }

                // Sort by binding index to match layout order
                entries.sort_by_key(|e| e.binding);

                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &entries,
                    label: Some(&format!("Bind Group for group: {}", group)),
                });
                bind_groups.push((group, bind_group));
            }

            // Sort by group index to match pipeline layout order
            bind_groups.sort_by_key(|(g, _)| *g);

            render_pass.set_pipeline(&cached.pipeline);
            for (i, (_, bind_group)) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }

            if let Some((refs, counts)) = cached_refs.as_ref() {
                render_pass.set_vertex_buffer(0, refs.vertex_buffer.slice(..));
                if let Some(instance_buffer) = &refs.instance_buffer {
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                }
                render_pass
                    .set_index_buffer(refs.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..counts.index_count, 0, 0..counts.instance_count);
            } else {
                // Fullscreen triangle fallback
                render_pass.draw(0..3, 0..1);
            }
        }

        // Return MSAA texture to pool if used
        if let Some(texture) = msaa_texture.take() {
            let key = TextureKey::new(size, fmt, sc, wgpu::TextureUsages::RENDER_ATTACHMENT);
            self.texture_pool.write().release(key, texture);
        }

        Ok(())
    }

    fn process_compute_pass(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        _pass: &PassObject,
    ) -> Result<(), RendererError> {
        Ok(()) // @TODO later
    }
}

fn create_bind_group_layouts(
    device: &wgpu::Device,
    uniforms: &HashMap<String, (u32, u32, Uniform)>,
) -> HashMap<u32, wgpu::BindGroupLayout> {
    let mut group_entries: HashMap<u32, Vec<wgpu::BindGroupLayoutEntry>> = HashMap::new();
    for (_, size, uniform) in uniforms.values() {
        if uniform.name.contains('.') {
            continue;
        }

        // WebGPU/Dawn is stricter about uniform binding sizes; ensure a safe minimum.
        let min_size = {
            let sz = *size as u64;
            let padded = wgpu::util::align_to(sz, 16).max(16);
            // padded is non-zero due to max(16)
            unsafe { std::num::NonZeroU64::new_unchecked(padded) }
        };

        // Decide entry type based on UniformData shape
        use crate::shader::uniform::UniformData;
        let entry = match &uniform.data {
            UniformData::Texture(meta) => {
                // Map naga metadata to wgpu binding types
                let view_dimension = match (meta.dim, meta.arrayed) {
                    (naga::ImageDimension::D2, false) => wgpu::TextureViewDimension::D2,
                    (naga::ImageDimension::D2, true) => wgpu::TextureViewDimension::D2Array,
                    (naga::ImageDimension::D3, _) => wgpu::TextureViewDimension::D3,
                    (naga::ImageDimension::Cube, false) => wgpu::TextureViewDimension::Cube,
                    (naga::ImageDimension::Cube, true) => wgpu::TextureViewDimension::CubeArray,
                    _ => wgpu::TextureViewDimension::D2,
                };
                match &meta.class {
                    naga::ImageClass::Depth { .. } => wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension,
                            multisampled: false,
                        },
                        count: None,
                    },
                    naga::ImageClass::Sampled { kind, multi } => {
                        let sample_type = match kind {
                            naga::ScalarKind::Sint => wgpu::TextureSampleType::Sint,
                            naga::ScalarKind::Uint => wgpu::TextureSampleType::Uint,
                            _ => wgpu::TextureSampleType::Float { filterable: true },
                        };
                        wgpu::BindGroupLayoutEntry {
                            binding: uniform.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type,
                                view_dimension,
                                multisampled: *multi,
                            },
                            count: None,
                        }
                    }
                    naga::ImageClass::Storage { format, access } => {
                        let access = *access;
                        let access = match access {
                            naga::StorageAccess::LOAD => wgpu::StorageTextureAccess::ReadOnly,
                            naga::StorageAccess::STORE => wgpu::StorageTextureAccess::WriteOnly,
                            _ => wgpu::StorageTextureAccess::ReadOnly,
                        };
                        let format = match format {
                            naga::StorageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
                            naga::StorageFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
                            naga::StorageFormat::R8Uint => wgpu::TextureFormat::R8Uint,
                            naga::StorageFormat::R8Sint => wgpu::TextureFormat::R8Sint,
                            naga::StorageFormat::R16Uint => wgpu::TextureFormat::R16Uint,
                            naga::StorageFormat::R16Sint => wgpu::TextureFormat::R16Sint,
                            naga::StorageFormat::R16Float => wgpu::TextureFormat::R16Float,
                            naga::StorageFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
                            naga::StorageFormat::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
                            naga::StorageFormat::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
                            naga::StorageFormat::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
                            naga::StorageFormat::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
                            naga::StorageFormat::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
                            naga::StorageFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
                            naga::StorageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
                            naga::StorageFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
                            naga::StorageFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
                            naga::StorageFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
                            naga::StorageFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
                            naga::StorageFormat::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
                            naga::StorageFormat::Rg11b10Ufloat => {
                                wgpu::TextureFormat::Rg11b10Ufloat
                            }
                            naga::StorageFormat::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
                            naga::StorageFormat::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
                            naga::StorageFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
                            naga::StorageFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
                            naga::StorageFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
                            naga::StorageFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
                            _ => wgpu::TextureFormat::Rgba8Unorm,
                        };
                        wgpu::BindGroupLayoutEntry {
                            binding: uniform.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::StorageTexture {
                                access,
                                format,
                                view_dimension,
                            },
                            count: None,
                        }
                    }
                }
            }
            UniformData::Sampler(_) => wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            UniformData::Storage(data) => {
                if let Some((_inner, span, access)) = data.first() {
                    let min = if *span == 0 { 16 } else { *span as u64 };
                    let min = unsafe { std::num::NonZeroU64::new_unchecked(min) };
                    wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: access.is_readonly(),
                            },
                            has_dynamic_offset: false,
                            min_binding_size: Some(min),
                        },
                        count: None,
                    }
                } else {
                    wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(min_size),
                        },
                        count: None,
                    }
                }
            }
            _ => wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(min_size),
                },
                count: None,
            },
        };

        group_entries.entry(uniform.group).or_default().push(entry);
    }

    let mut layouts = HashMap::new();
    for (group, entries) in group_entries {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Bind Group Layout for group: {}", group)),
            entries: entries.as_slice(),
        });
        layouts.insert(group, layout);
    }

    layouts
}

fn create_render_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &HashMap<u32, wgpu::BindGroupLayout>,
    shader: &crate::ShaderObject,
    format: wgpu::TextureFormat,
    sample_count: u32,
    vertex_layouts: Option<(
        &wgpu::VertexBufferLayout<'static>,
        Option<&wgpu::VertexBufferLayout<'static>>,
    )>,
) -> wgpu::RenderPipeline {
    let mut vs_entry = None;
    let mut fs_entry = None;
    for entry_point in shader.module.entry_points.iter() {
        if entry_point.stage == naga::ShaderStage::Vertex {
            vs_entry = entry_point.function.name.clone();
        }
        if entry_point.stage == naga::ShaderStage::Fragment {
            fs_entry = entry_point.function.name.clone();
        }
    }

    let module = Cow::Owned(shader.module.clone());
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Naga(module),
    });

    let mut sorted_groups: Vec<_> = bind_group_layouts.keys().collect();
    sorted_groups.sort();
    let mut bind_group_layouts_sorted: Vec<&wgpu::BindGroupLayout> = Vec::new();
    for g in sorted_groups.into_iter() {
        if let Some(l) = bind_group_layouts.get(g) {
            bind_group_layouts_sorted.push(l);
        }
    }

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Default Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts_sorted,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Default Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some(vs_entry.as_deref().unwrap_or("vs_main")),
            buffers: {
                // Build a local Vec to keep the layouts alive for this call
                let mut tmp: Vec<wgpu::VertexBufferLayout> = Vec::new();
                if let Some((v, instance_buffer)) = vertex_layouts {
                    tmp.push(v.clone());
                    if let Some(i) = instance_buffer {
                        tmp.push(i.clone());
                    }
                }
                Box::leak(tmp.into_boxed_slice())
            },
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some(fs_entry.as_deref().unwrap_or("fs_main")),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                // @TODO implement more granular control over blending
                //
                // Linear Interpolation Formula: Fa*Fc + (1-Fa)*Bc
                //
                // In wgpu:
                // FACTOR * Src (OPERATION) FACTOR * Dst
                //
                // Where:
                // Src is the Foreground (image on top)
                // Dst is the Background (image on bottom)
                //
                // FACTOR can be:
                //   /// 0.0
                //   Zero = 0,
                //   /// 1.0
                //   One = 1,
                //   /// S.color
                //   Src = 2,
                //   /// 1.0 - S.color
                //   OneMinusSrc = 3,
                //   /// S.alpha
                //   SrcAlpha = 4,
                //   /// 1.0 - S.alpha
                //   OneMinusSrcAlpha = 5,
                //   /// D.color
                //   Dst = 6,
                //   /// 1.0 - D.color
                //   OneMinusDst = 7,
                //   /// D.alpha
                //   DstAlpha = 8,
                //   /// 1.0 - D.alpha
                //   OneMinusDstAlpha = 9,
                //   /// min(S.alpha, 1.0 - D.alpha)
                //   SrcAlphaSaturated = 10,
                //   /// Constant
                //   Constant = 11,
                //   /// 1.0 - Constant
                //   OneMinusConstant = 12,
                //   /// S1.color
                //   Src1 = 13,
                //   /// 1.0 - S1.color
                //   OneMinusSrc1 = 14,
                //   /// S1.alpha
                //   Src1Alpha = 15,
                //   /// 1.0 - S1.alpha
                //   OneMinusSrc1Alpha = 16,
                //
                // OPERATION Can be:
                //   /// Src + Dst
                //   #[default]
                //   Add = 0,
                //   /// Src - Dst
                //   Subtract = 1,
                //   /// Dst - Src
                //   ReverseSubtract = 2,
                //   /// min(Src, Dst)
                //   Min = 3,
                //   /// max(Src, Dst)
                //   Max = 4,
                //
                // blend: Some(wgpu::BlendState {
                //     color: wgpu::BlendComponent {
                //         src_factor: wgpu::BlendFactor::One,
                //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                //         operation: wgpu::BlendOperation::Add,
                //     },
                //     alpha: wgpu::BlendComponent {
                //         src_factor: wgpu::BlendFactor::One,
                //         dst_factor: wgpu::BlendFactor::Zero,
                //         operation: wgpu::BlendOperation::Add,
                //     },
                // }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

#[cfg(test)]
mod more_error_path_tests {
    use super::*;

    // Story: Creating a texture from RGBA8 raw bytes with explicit size succeeds and yields expected size.
    #[test]
    fn create_texture_with_raw_bytes_success() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            // 2x2 RGBA (4 pixels): solid colors
            #[rustfmt::skip]
            let bytes: [u8; 16] = [
                255, 0, 0, 255,   0, 255, 0, 255,
                0, 0, 255, 255,   255, 255, 255, 255,
            ];
            let tex = renderer
                .create_texture_with(&bytes[..], crate::Size::from((2u32, 2u32)))
                .await
                .expect("texture raw bytes");
            let sz = tex.size();
            assert_eq!([sz.width, sz.height], [2, 2]);
        });
    }

    // Story: Creating a texture from invalid raw bytes with explicit format/size returns an error.
    #[test]
    fn create_texture_with_invalid_raw_bytes_errors() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let bad = [1u8, 2, 3];
            let res = renderer
                .create_texture_with(
                    &bad[..],
                    TextureOptions::from(crate::Size::from((2u32, 2u32))),
                )
                .await;
            assert!(res.is_err(), "expected error for insufficient raw bytes");
        });
    }

    // Story: Creating a texture from a non-existent file path returns an error wrapped by InitializationError.
    #[test]
    fn create_texture_from_nonexistent_path_errors() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let p = std::path::PathBuf::from("/path/does/not/exist.png");
            let res = renderer
                .create_texture_with(&p, TextureOptions::from(crate::Size::from((1u32, 1u32))))
                .await;
            assert!(res.is_err());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::TargetFrame;
    use crate::{Pass, Shader};

    struct TestFrame {
        view: wgpu::TextureView,
        format: wgpu::TextureFormat,
    }

    impl TargetFrame for TestFrame {
        fn view(&self) -> &wgpu::TextureView {
            &self.view
        }
        fn format(&self) -> wgpu::TextureFormat {
            self.format
        }
        fn present(self: Box<Self>) {}
        fn auto_present(&self) -> bool {
            false
        }
    }

    async fn create_device_and_queue() -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let instance = platform::all::create_instance().await;
        let adapter = platform::all::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = platform::all::request_device(&adapter)
            .await
            .expect("device");
        (adapter, device, queue)
    }

    #[test]
    fn render_with_texture_sampler_smoke() {
        pollster::block_on(async move {
            // Create renderer and a small texture target
            let (_adapter, _device, _queue) = create_device_and_queue().await;
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            // Simple WGSL sampling a texture
            let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");

            // Create a tiny 2x2 RGBA image and upload as texture
            #[rustfmt::skip]
            let pixels: Vec<u8> = vec![
                255, 0, 0,   255,    0,   255, 0,   255,
                0,   0, 255, 255,    255, 255, 255, 255,
            ];
            let size = crate::Size::from((2u32, 2u32));
            let tex = renderer
                .create_texture_with(&pixels, size)
                .await
                .expect("texture");
            shader.set("tex", &tex).expect("set tex");

            // Render should succeed without panicking
            renderer.render(&shader, &target).expect("render ok");

            let image: Vec<u8> = target.get_image();

            assert_eq!(image.len(), 8 * 8 * 4);
        });
    }

    // Story: Render a simple mesh with position-only vertices using a minimal shader.
    #[test]
    fn render_with_mesh_positions_only() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("mesh", &shader);

            // Build a triangle mesh
            let mut mesh = crate::mesh::Mesh::new();
            use crate::mesh::{Position, Vertex};
            mesh.add_vertices([
                Vertex::from_position(Position::Pos3([-0.5, -0.5, 0.0])),
                Vertex::from_position(Position::Pos3([0.5, -0.5, 0.0])),
                Vertex::from_position(Position::Pos3([0.0, 0.5, 0.0])),
            ]);
            pass.add_mesh(&mesh);

            renderer.render(&pass, &target).expect("render ok");
        });
    }

    fn create_test_frame(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> TestFrame {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Resolve Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        // Keep texture alive by capturing it in the view's lifetime (wgpu keeps texture alive internally)
        // Explicit binding not necessary here.
        TestFrame { view, format }
    }

    #[test]
    fn sample_count_set_and_get() {
        pollster::block_on(async move {
            let (_adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            assert_eq!(ctx.sample_count(), 1);
            ctx.set_sample_count(0);
            assert_eq!(ctx.sample_count(), 1);
            ctx.set_sample_count(3);
            assert_eq!(ctx.sample_count(), 3);
            ctx.set_sample_count(4);
            assert_eq!(ctx.sample_count(), 4);
        });
    }

    #[test]
    fn pipeline_cache_keyed_by_sample_count() {
        pollster::block_on(async move {
            let (adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            let fmt = wgpu::TextureFormat::Rgba8Unorm;
            let size = wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            };
            let frame = create_test_frame(&ctx.device, size, fmt);

            let pass = Pass::from_shader("single", &Shader::default());
            let pass = pass.passes().into_iter().next().expect("pass");
            let shader_hash = pass
                .shaders
                .read()
                .first()
                .map(|shader| shader.hash)
                .expect("shader hash");

            // sc = 1
            ctx.set_sample_count(1);
            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Test Encoder 1"),
                });
            ctx.process_render_pass(&mut encoder, pass, &frame, size)
                .expect("render pass sc=1");
            assert!(ctx.render_pipelines.contains_key(&(shader_hash, fmt, 1, 0)));

            // pick a supported msaa (>1) if any
            let flags = adapter.get_texture_format_features(fmt).flags;
            let sc2 = if flags.sample_count_supported(4) {
                4
            } else if flags.sample_count_supported(2) {
                2
            } else {
                1
            };

            if sc2 > 1 {
                ctx.set_sample_count(sc2);
                let mut encoder2 =
                    ctx.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Test Encoder 2"),
                        });
                ctx.process_render_pass(&mut encoder2, pass, &frame, size)
                    .expect("render pass sc>1");
                assert!(
                    ctx.render_pipelines
                        .contains_key(&(shader_hash, fmt, sc2, 0))
                );
                // Ensure entries are distinct keys
                assert_ne!(sc2, 1);
            }
        });
    }

    #[test]
    fn pick_sample_count_properties() {
        pollster::block_on(async move {
            let (adapter, _device, _queue) = create_device_and_queue().await;
            let fmt = wgpu::TextureFormat::Rgba8Unorm;

            // wanted = 0 -> 1
            let s0 = platform::all::pick_sample_count(&adapter, 0, fmt);
            assert_eq!(s0, 1);

            // wanted = 5 -> power of two <= 5 and supported or halved down to supported
            let s5 = platform::all::pick_sample_count(&adapter, 5, fmt);
            assert!((1..=5).contains(&s5));
            assert!(s5.is_power_of_two());
        });
    }

    // Story: try_get_frame_with_retry retries once on Lost/Outdated and returns other errors as-is.
    #[test]
    fn try_get_frame_with_retry_exercises_paths() {
        use std::collections::VecDeque;
        struct DummyFrame;
        impl crate::TargetFrame for DummyFrame {
            fn view(&self) -> &wgpu::TextureView {
                panic!("not used")
            }
            fn format(&self) -> wgpu::TextureFormat {
                wgpu::TextureFormat::Rgba8Unorm
            }
            fn present(self: Box<Self>) {}
            fn auto_present(&self) -> bool {
                false
            }
        }
        struct DummyTarget {
            size: crate::Size,
            seq: parking_lot::RwLock<
                VecDeque<Result<Box<dyn crate::TargetFrame>, wgpu::SurfaceError>>,
            >,
        }
        impl DummyTarget {
            fn new(seq: Vec<Result<Box<dyn crate::TargetFrame>, wgpu::SurfaceError>>) -> Self {
                Self {
                    size: crate::Size::from((1u32, 1u32)),
                    seq: parking_lot::RwLock::new(seq.into()),
                }
            }
        }
        impl crate::Target for DummyTarget {
            fn size(&self) -> crate::Size {
                self.size
            }
            fn resize(&mut self, _s: impl Into<crate::Size>) {}
            fn get_current_frame(&self) -> Result<Box<dyn crate::TargetFrame>, wgpu::SurfaceError> {
                self.seq
                    .write()
                    .pop_front()
                    .unwrap_or_else(|| Ok(Box::new(DummyFrame)))
            }
            fn get_image(&self) -> Vec<u8> {
                Vec::new()
            }
        }

        pollster::block_on(async move {
            let (_adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            // Case 1: Lost then Ok -> success
            let t1 = DummyTarget::new(vec![
                Err(wgpu::SurfaceError::Lost),
                Ok(Box::new(DummyFrame)),
            ]);
            let f1 = ctx.try_get_frame_with_retry(&t1);
            assert!(f1.is_ok());

            // Case 2: OutOfMemory -> error returned
            let t2 = DummyTarget::new(vec![Err(wgpu::SurfaceError::OutOfMemory)]);
            let f2 = ctx.try_get_frame_with_retry(&t2);
            assert!(matches!(f2, Err(wgpu::SurfaceError::OutOfMemory)));

            // Case 3: Outdated then Timeout -> returns second error
            let t3 = DummyTarget::new(vec![
                Err(wgpu::SurfaceError::Outdated),
                Err(wgpu::SurfaceError::Timeout),
            ]);
            let f3 = ctx.try_get_frame_with_retry(&t3);
            assert!(matches!(f3, Err(wgpu::SurfaceError::Timeout)));
        });
    }
}
