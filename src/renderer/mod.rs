use crate::{
    PassObject, ShaderHash, Target, TargetFrame, TextureInput, TextureOptions, TextureTarget,
    UniformData,
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
enum PushMode {
    // Use native push constants (only supported on native, single root for now)
    Native {
        root: String,
        size: u32,
    },
    // Fallback to uniforms (Web or size/multi-root on native)
    // Map variable name -> binding number within the fallback group
    Fallback {
        group: u32,
        bindings: std::collections::HashMap<String, u32>,
    },
}

#[derive(Debug)]
struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    // Map of bind group index -> layout (keeps group indices stable)
    bind_group_layouts: std::collections::HashMap<u32, wgpu::BindGroupLayout>,
    push_mode: Option<PushMode>,
}

#[derive(Debug)]
struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layouts: std::collections::HashMap<u32, wgpu::BindGroupLayout>,
}

#[derive(Debug, Default)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(wasm, wasm_bindgen)]
#[lsp_doc("docs/api/core/renderer/renderer.md")]
pub struct Renderer {
    instance: RwLock<Option<Arc<wgpu::Instance>>>,
    adapter: RwLock<Option<wgpu::Adapter>>,
    context: RwLock<Option<Arc<RenderContext>>>,
}

crate::impl_fc_kind!(Renderer, "Renderer");

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
        // For offscreen texture targets on Web, force sample_count = 1 to avoid MSAA resolve issues.
        // Canvas targets may still use MSAA via the surface path.
        context.set_sample_count(1);
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
                let object = crate::TextureObject::from_file(context.as_ref(), path)?;
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
        format: impl Into<crate::TextureFormat>,
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
            format.into().into(),
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
        let adapter_ref = adapter.as_ref().ok_or(InitializationError::AdapterNotSet)?;
        let config = configure_surface(&context.device, adapter_ref, &surface, &size);

        // Negotiate and store effective sample count (currently default wanted=1; configurable later)
        if let Some(adapter_ref) = adapter.as_ref() {
            let sample_count = platform::all::pick_sample_count(adapter_ref, 1, config.format);
            context.set_sample_count(sample_count);
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

/// Key for caching render pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RenderPipelineKey {
    shader_hash: ShaderHash,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    sample_count: u32,
}

// Key for caching compute pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComputePipelineKey {
    shader_hash: ShaderHash,
    layout_signature: u64,
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

    // Cache for RenderPipelines and ComputePipelines
    render_pipelines: DashMap<RenderPipelineKey, RenderPipeline>,
    compute_pipelines: DashMap<ComputePipelineKey, ComputePipeline>,

    buffer_pool: RwLock<UniformBufferPool>,
    // Storage buffer pool (STORAGE | COPY_DST), separate from uniform pool
    storage_pool: RwLock<StorageBufferPool>,

    pub(crate) readback_pool: RwLock<ReadbackBufferPool>,
    pub(crate) texture_pool: RwLock<TexturePool>,

    // Texture registry: id -> TextureObject
    textures: DashMap<crate::texture::TextureId, Arc<crate::TextureObject>>,
    next_id: AtomicU64,

    // Persistent storage buffer registry: root name -> (buffer, span)
    storage_registry: DashMap<String, (wgpu::Buffer, u64)>,

    // MSAA sample count negotiated for current target/format
    sample_count: AtomicU32,
}

impl RenderContext {
    /// Creates a new Context with the given device and queue.
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let buffer_pool = UniformBufferPool::new("Uniform Buffer Pool", &device);
        let storage_pool = StorageBufferPool::new("Storage Buffer Pool", &device);
        let readback_pool = ReadbackBufferPool::new("Readback Buffer Pool", 8);

        RenderContext {
            device,
            queue,

            render_pipelines: DashMap::new(),
            compute_pipelines: DashMap::new(),

            buffer_pool: RwLock::new(buffer_pool),
            storage_pool: RwLock::new(storage_pool),
            readback_pool: RwLock::new(readback_pool),
            texture_pool: RwLock::new(TexturePool::new(16)),

            textures: DashMap::new(),
            next_id: AtomicU64::new(1),
            storage_registry: DashMap::new(),
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

        let frame = self.try_get_frame_with_retry(target)?;

        let pass_list = renderable.passes();
        for pass in pass_list.iter() {
            let pass = pass.as_ref();
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
    pub(crate) fn register_texture(
        &self,
        texture: Arc<crate::TextureObject>,
    ) -> crate::texture::TextureId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let texture_id = crate::texture::TextureId(id);
        self.textures.insert(texture_id, texture);
        texture_id
    }

    pub(crate) fn get_texture(
        &self,
        id: &crate::texture::TextureId,
    ) -> Option<Arc<crate::TextureObject>> {
        self.textures.get(id).map(|texture| texture.clone())
    }

    pub(crate) fn set_sample_count(&self, count: u32) {
        self.sample_count.store(count.max(1), Ordering::Relaxed);
    }

    pub(crate) fn sample_count(&self) -> u32 {
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

        // Choose color attachment: per-pass override or default frame view.
        // Offscreen color targets are bound single-sampled; surface targets may use MSAA.
        let mut sample_count = self.sample_count();
        let mut resolve_target: Option<&wgpu::TextureView> = None;

        let mut color_format = frame.format();
        let mut pass_texture_view = None;
        if let Some(id) = *pass.color_target.read() {
            if let Some(texture) = self.get_texture(&id) {
                pass_texture_view = Some(texture.create_view());
                color_format = texture.format();
                sample_count = texture.inner.sample_count();
            } else {
                return Err(RendererError::TextureNotFoundError(id));
            }
        }

        let mut texture_view: &wgpu::TextureView =
            if let Some(pass_view) = pass_texture_view.as_ref() {
                pass_view
            } else {
                frame.view()
            };

        // Keep MSAA resources alive for the duration of the pass
        let mut msaa_texture: Option<wgpu::Texture> = None;
        let mut _msaa_view: Option<wgpu::TextureView> = None;
        if pass_texture_view.is_none() && sample_count > 1 {
            let key = TextureKey::new(
                size,
                color_format,
                sample_count,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let texture = {
                let mut pool = self.texture_pool.write();
                pool.acquire(&self.device, key)
            };
            _msaa_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            msaa_texture = Some(texture);
            texture_view = match _msaa_view.as_ref() {
                Some(v) => v,
                None => return Err(RendererError::MsaaViewMissing),
            };
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

        // Optional depth attachment if the pass has a depth target
        let (depth_view, depth_format, depth_sc_opt) =
            if let Some(depth_id) = *pass.depth_target.read() {
                if let Some(texture) = self.get_texture(&depth_id) {
                    let depth_view = texture.create_view();
                    let depth_format = texture.format();
                    let depth_sc = texture.inner.sample_count();
                    (Some(depth_view), Some(depth_format), Some(depth_sc))
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

        // Validate depth sample count matches the pass sample count, if present
        if let Some(sc) = depth_sc_opt
            && sc != sample_count
        {
            return Err(RendererError::DepthSampleCountMismatch {
                depth: sc,
                pass: sample_count,
            });
        }

        let depth_stencil_attachment =
            depth_view
                .as_ref()
                .map(|depth_view| wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                });

        let descriptor = wgpu::RenderPassDescriptor {
            label: Some(&format!("Render Pass: {}", pass.name.clone())),
            color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        let required_size = *pass.required_buffer_size.read();
        self.buffer_pool
            .write()
            .ensure_capacity(required_size, &self.device);

        for shader in pass.shaders.read().iter() {
            shader.flush_pending();
            let shader_meshes = shader.meshes.read().clone();
            let vertex_buffer_layouts = if let Some(first_mesh) = shader_meshes.first() {
                // Ensure schemas are derived (and buffers packed) before pipeline creation
                first_mesh.vertex_buffers(&self.device, &self.queue)?;
                // Now derive vertex buffer layouts; propagate any mapping error
                create_vertex_buffer_layouts(shader, first_mesh.as_ref())?
            } else {
                None
            };

            let pipeline_key = RenderPipelineKey {
                shader_hash: shader.hash,
                color_format,
                depth_format,
                sample_count,
            };
            let cached_pipeline = self
                .render_pipelines
                .entry(pipeline_key)
                .or_insert_with(|| {
                    create_render_pipeline(
                        &self.device,
                        shader,
                        color_format,
                        sample_count,
                        vertex_buffer_layouts,
                        depth_format,
                    )
                });

            // Collect resources per bind group to build entries safely with owned views/samplers
            #[derive(Default)]
            struct BindGroupResources {
                uniform_buffers: Vec<(u32, buffer_pool::BufferLocation)>,
                // legacy storage-pool backed buffers (kept for compatibility in other paths)
                storage_buffers: Vec<(u32, buffer_pool::BufferLocation)>,
                // persistent storage buffers (buffer, span)
                persistent_storage_buffers: Vec<(u32, wgpu::Buffer, u64)>,
                last_texture_sampler: Option<wgpu::Sampler>,
                samplers: Vec<(u32, wgpu::Sampler)>,
                views: Vec<(u32, wgpu::TextureView)>,
            }

            let mut groups: HashMap<u32, BindGroupResources> = HashMap::new();
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                match &uniform.data {
                    UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = tex.create_view();
                            let sampler = tex.sampler();
                            let group_entry = groups.entry(uniform.group).or_default();
                            group_entry.views.push((uniform.binding, view));
                            group_entry.last_texture_sampler = Some(sampler);
                        } else {
                            log::warn!(
                                "Texture handle {:?} not found for uniform {}",
                                meta.id,
                                name
                            );
                        }
                    }
                    UniformData::Sampler(info) => {
                        // Create the appropriate sampler type explicitly so it always matches the layout
                        let sampler = if info.comparison {
                            self.device.create_sampler(&wgpu::SamplerDescriptor {
                                label: Some("Default Comparison Sampler"),
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Linear,
                                lod_min_clamp: 0.0,
                                lod_max_clamp: 100.0,
                                compare: Some(wgpu::CompareFunction::LessEqual),
                                anisotropy_clamp: 1,
                                border_color: None,
                            })
                        } else {
                            self.device
                                .create_sampler(&wgpu::SamplerDescriptor::default())
                        };
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .samplers
                            .push((uniform.binding, sampler));
                    }
                    UniformData::Storage(data) => {
                        if let Some((_inner, span, _access)) = data.first() {
                            // Acquire persistent storage buffer and upload only if necessary
                            let span_u64 = *span as u64;
                            // Obtain initial bytes for creation or update
                            let init_bytes: Vec<u8> = {
                                let s = shader.storage.try_read().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage read".into())
                                })?;
                                s.get_bytes(name)
                                    .map(|b| b.to_vec())
                                    .unwrap_or_else(|| vec![0u8; span_u64 as usize])
                            };
                            // Create or get persistent buffer
                            let buf = {
                                // Check existing
                                if let Some(entry) = self.storage_registry.get(name) {
                                    let (existing, existing_span) = entry.value();
                                    if *existing_span != span_u64 {
                                        drop(entry);
                                        // Recreate with new span
                                        let buffer =
                                            self.device.create_buffer(&wgpu::BufferDescriptor {
                                                label: Some(&format!(
                                                    "Persistent Storage Buffer: {}",
                                                    name
                                                )),
                                                size: span_u64,
                                                usage: wgpu::BufferUsages::STORAGE
                                                    | wgpu::BufferUsages::COPY_DST
                                                    | wgpu::BufferUsages::COPY_SRC,
                                                mapped_at_creation: false,
                                            });
                                        self.queue.write_buffer(&buffer, 0, &init_bytes);
                                        self.storage_registry
                                            .insert(name.to_string(), (buffer.clone(), span_u64));
                                        buffer
                                    } else {
                                        existing.clone()
                                    }
                                } else {
                                    // Create
                                    let buffer =
                                        self.device.create_buffer(&wgpu::BufferDescriptor {
                                            label: Some(&format!(
                                                "Persistent Storage Buffer: {}",
                                                name
                                            )),
                                            size: span_u64,
                                            usage: wgpu::BufferUsages::STORAGE
                                                | wgpu::BufferUsages::COPY_DST
                                                | wgpu::BufferUsages::COPY_SRC,
                                            mapped_at_creation: false,
                                        });
                                    self.queue.write_buffer(&buffer, 0, &init_bytes);
                                    self.storage_registry
                                        .insert(name.to_string(), (buffer.clone(), span_u64));
                                    buffer
                                }
                            };
                            // If CPU blob is marked dirty, upload and clear flag
                            let need_upload = {
                                let s = shader.storage.try_read().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage read".into())
                                })?;
                                s.is_storage_dirty(name)
                            };
                            if need_upload {
                                let bytes: Vec<u8> = {
                                    let s = shader.storage.try_read().ok_or_else(|| {
                                        crate::shader::ShaderError::Busy("storage read".into())
                                    })?;
                                    s.get_bytes(name)
                                        .map(|b| b.to_vec())
                                        .unwrap_or_else(|| vec![0u8; span_u64 as usize])
                                };
                                self.queue.write_buffer(&buf, 0, &bytes);
                                let mut s = shader.storage.try_write().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage write".into())
                                })?;
                                s.clear_storage_dirty(name);
                            }

                            groups
                                .entry(uniform.group)
                                .or_default()
                                .persistent_storage_buffers
                                .push((uniform.binding, buf, span_u64));
                        } else {
                            return Err(crate::ShaderError::UniformNotFound(name.clone()).into());
                        }
                    }
                    UniformData::PushConstant(_) => {
                        // Fallback mode: each push constant root became a classic uniform buffer in rewritten module.
                        if let Some(PushMode::Fallback { group, bindings }) =
                            &cached_pipeline.push_mode
                            && let Some(binding) = bindings.get(name)
                        {
                            let storage = shader.storage.read();
                            let bytes = storage
                                .get_bytes(name)
                                .ok_or(crate::ShaderError::UniformNotFound(name.clone()))?;

                            let buffer_location = {
                                let mut buffer_pool = self.buffer_pool.write();
                                buffer_pool.upload(bytes, &self.queue, &self.device)
                            };

                            groups
                                .entry(*group)
                                .or_default()
                                .uniform_buffers
                                .push((*binding, buffer_location));
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
                            .uniform_buffers
                            .push((uniform.binding, buffer_location));
                    }
                }
            }

            // Build bind groups per layout (by group index)
            let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
            use std::collections::HashSet;
            let mut present_groups: HashSet<u32> = HashSet::new();

            for (group_index, resources) in groups.into_iter() {
                let Some(layout) = cached_pipeline.bind_group_layouts.get(&group_index) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group_index
                    )));
                };

                // Assemble entries borrowing from owned resources and buffer pool
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
                for (binding, view) in resources.views.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }
                for (binding, sampler) in resources.samplers.iter() {
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
                    label: Some(&format!("Bind Group for group: {}", group_index)),
                });
                present_groups.insert(group_index);
                bind_groups.push((group_index, bind_group));
            }

            // Ensure empty bind groups are set for placeholder layouts the pipeline expects
            for (group, layout) in cached_pipeline.bind_group_layouts.iter() {
                if !present_groups.contains(group) {
                    // Create an empty bind group (layout is expected to have zero entries for placeholders)
                    let empty = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &[],
                        label: Some(&format!("Empty Bind Group for group: {}", group)),
                    });
                    bind_groups.push((*group, empty));
                }
            }

            // Sort by group index to match pipeline layout order
            bind_groups.sort_by_key(|(g, _)| *g);
            render_pass.set_pipeline(&cached_pipeline.pipeline);
            for (group, bind_group) in bind_groups.iter() {
                render_pass.set_bind_group(*group, bind_group, &[]);
            }

            // Set native push constants just before draw if applicable
            if let Some(PushMode::Native { root, .. }) = &cached_pipeline.push_mode {
                let storage = shader
                    .storage
                    .try_read()
                    .ok_or_else(|| crate::shader::ShaderError::Busy("storage read".into()))?;
                if let Some(bytes) = storage.get_bytes(root) {
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        0,
                        bytes,
                    );
                }
            }

            match shader_meshes.len() {
                0 => render_pass.draw(0..3, 0..1),
                _ => {
                    for mesh_object in shader_meshes.iter() {
                        let (refs, counts) =
                            mesh_object.vertex_buffers(&self.device, &self.queue)?;
                        render_pass.set_vertex_buffer(0, refs.vertex_buffer.slice(..));
                        if let Some(instance_buffer) = &refs.instance_buffer {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        }
                        render_pass.set_index_buffer(
                            refs.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(
                            0..counts.index_count,
                            0,
                            0..counts.instance_count,
                        );
                    }
                }
            }
        }

        // Return MSAA texture to pool if used
        if let Some(texture) = msaa_texture.take() {
            let key = TextureKey::new(
                size,
                color_format,
                sample_count,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            self.texture_pool.write().release(key, texture);
        }

        Ok(())
    }

    fn process_compute_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
    ) -> Result<(), RendererError> {
        // Similar resource binding as render pass, but no target
        self.buffer_pool.write().reset();
        self.storage_pool.write().reset();

        for (shader_index, shader) in pass.shaders.read().iter().enumerate() {
            shader.flush_pending();
            // Build or fetch pipeline
            // Layout signature: hash the bind group layouts
            let layouts = create_bind_group_layouts(&self.device, shader);

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            for (g, l) in layouts.iter() {
                hasher.update(g.to_le_bytes());
                // Hash the address as a proxy (layout contents are derived from uniforms)
                let ptr = (l as *const _ as usize) as u64;
                hasher.update(ptr.to_le_bytes());
            }
            let sig_bytes = hasher.finalize();
            let sig = u64::from_le_bytes([
                sig_bytes[0],
                sig_bytes[1],
                sig_bytes[2],
                sig_bytes[3],
                sig_bytes[4],
                sig_bytes[5],
                sig_bytes[6],
                sig_bytes[7],
            ]);

            let cached = self
                .compute_pipelines
                .entry(ComputePipelineKey {
                    shader_hash: shader.hash,
                    layout_signature: sig,
                })
                .or_insert_with(|| {
                    let mut sorted_groups: Vec<_> = layouts.keys().cloned().collect();
                    sorted_groups.sort();
                    let mut bind_group_layouts_sorted: Vec<&wgpu::BindGroupLayout> = Vec::new();
                    for g in sorted_groups.into_iter() {
                        if let Some(l) = layouts.get(&g) {
                            bind_group_layouts_sorted.push(l);
                        }
                    }

                    let layout =
                        self.device
                            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: Some("Compute Pipeline Layout"),
                                bind_group_layouts: &bind_group_layouts_sorted,
                                push_constant_ranges: &[],
                            });

                    let module = Cow::Owned(shader.module.clone());
                    let shader_module =
                        self.device
                            .create_shader_module(wgpu::ShaderModuleDescriptor {
                                label: Some("Compute Shader"),
                                source: wgpu::ShaderSource::Naga(module),
                            });

                    // Find compute entry point name
                    let mut cs_entry = None;
                    for ep in shader.module.entry_points.iter() {
                        if ep.stage == naga::ShaderStage::Compute {
                            cs_entry = ep.function.name.clone();
                            break;
                        }
                    }

                    let pipeline =
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some("Compute Pipeline"),
                                layout: Some(&layout),
                                module: &shader_module,
                                entry_point: Some(cs_entry.as_deref().unwrap_or("cs_main")),
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                                cache: None,
                            });

                    ComputePipeline {
                        pipeline,
                        bind_group_layouts: layouts,
                    }
                });
            #[derive(Default)]
            struct BindGroupResources {
                buffers: Vec<(u32, buffer_pool::BufferLocation)>,
                storage_buffers: Vec<(u32, wgpu::Buffer, u64)>,
                views: Vec<(u32, wgpu::TextureView)>,
                samplers: Vec<(u32, wgpu::Sampler)>,
            }
            let mut groups: HashMap<u32, BindGroupResources> = HashMap::new();
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;
                match &uniform.data {
                    UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = tex.create_view();
                            groups
                                .entry(uniform.group)
                                .or_default()
                                .views
                                .push((uniform.binding, view));
                        }
                    }
                    UniformData::Sampler(info) => {
                        let sampler = if info.comparison {
                            self.device.create_sampler(&wgpu::SamplerDescriptor {
                                label: Some("Default Comparison Sampler"),
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Linear,
                                lod_min_clamp: 0.0,
                                lod_max_clamp: 100.0,
                                compare: Some(wgpu::CompareFunction::LessEqual),
                                anisotropy_clamp: 1,
                                border_color: None,
                            })
                        } else {
                            self.device
                                .create_sampler(&wgpu::SamplerDescriptor::default())
                        };
                        groups
                            .entry(uniform.group)
                            .or_default()
                            .samplers
                            .push((uniform.binding, sampler));
                    }
                    UniformData::Storage(data) => {
                        if let Some((_inner, span_u32, _access)) = data.first() {
                            let span = *span_u32 as u64;
                            // Obtain bytes
                            let init_bytes: Vec<u8> = {
                                let s = shader.storage.try_read().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage read".into())
                                })?;
                                s.get_bytes(name)
                                    .map(|b| b.to_vec())
                                    .unwrap_or_else(|| vec![0u8; span as usize])
                            };
                            // Create or get buffer
                            let buf = {
                                if let Some(entry) = self.storage_registry.get(name) {
                                    let (existing, existing_span) = entry.value();
                                    if *existing_span != span {
                                        drop(entry);
                                        let buffer =
                                            self.device.create_buffer(&wgpu::BufferDescriptor {
                                                label: Some(&format!(
                                                    "Persistent Storage Buffer: {}",
                                                    name
                                                )),
                                                size: span,
                                                usage: wgpu::BufferUsages::STORAGE
                                                    | wgpu::BufferUsages::COPY_DST
                                                    | wgpu::BufferUsages::COPY_SRC,
                                                mapped_at_creation: false,
                                            });
                                        self.queue.write_buffer(&buffer, 0, &init_bytes);
                                        self.storage_registry
                                            .insert(name.to_string(), (buffer.clone(), span));
                                        buffer
                                    } else {
                                        existing.clone()
                                    }
                                } else {
                                    let buffer =
                                        self.device.create_buffer(&wgpu::BufferDescriptor {
                                            label: Some(&format!(
                                                "Persistent Storage Buffer: {}",
                                                name
                                            )),
                                            size: span,
                                            usage: wgpu::BufferUsages::STORAGE
                                                | wgpu::BufferUsages::COPY_DST
                                                | wgpu::BufferUsages::COPY_SRC,
                                            mapped_at_creation: false,
                                        });
                                    self.queue.write_buffer(&buffer, 0, &init_bytes);
                                    self.storage_registry
                                        .insert(name.to_string(), (buffer.clone(), span));
                                    buffer
                                }
                            };
                            // Upload if dirty
                            let need_upload = {
                                let s = shader.storage.try_read().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage read".into())
                                })?;
                                s.is_storage_dirty(name)
                            };
                            if need_upload {
                                let bytes: Vec<u8> = {
                                    let s = shader.storage.try_read().ok_or_else(|| {
                                        crate::shader::ShaderError::Busy("storage read".into())
                                    })?;
                                    s.get_bytes(name)
                                        .map(|b| b.to_vec())
                                        .unwrap_or_else(|| vec![0u8; span as usize])
                                };
                                self.queue.write_buffer(&buf, 0, &bytes);
                                let mut s = shader.storage.try_write().ok_or_else(|| {
                                    crate::shader::ShaderError::Busy("storage write".into())
                                })?;
                                s.clear_storage_dirty(name);
                            }
                            groups
                                .entry(uniform.group)
                                .or_default()
                                .storage_buffers
                                .push((uniform.binding, buf, span));
                        }
                    }
                    _ => {
                        let storage = shader.storage.try_read().ok_or_else(|| {
                            crate::shader::ShaderError::Busy("storage read".into())
                        })?;
                        let bytes = storage
                            .get_bytes(name)
                            .ok_or(crate::ShaderError::UniformNotFound(name.clone()))?;
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
            for (group, owned) in groups.into_iter() {
                let Some(layout) = cached.bind_group_layouts.get(&group) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group
                    )));
                };
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
                for (binding, sampler) in owned.samplers.iter() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    });
                }
                entries.sort_by_key(|e| e.binding);
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &entries,
                    label: Some(&format!("Compute Bind Group for group: {}", group)),
                });
                bind_groups.push((group, bind_group));
            }
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

            let (x, y, z) = pass.compute_dispatch();
            ccompute_pass.dispatch_workgroups(x, y, z);
        }

        Ok(())
    }
}

type VertexLayouts = (
    wgpu::VertexBufferLayout<'static>,
    Option<wgpu::VertexBufferLayout<'static>>,
);

// @TODO the Shader should provide this
//.      - heck if the logic in the Shader is the same
//.      - Create and persist the layouts in the ShaderObject
fn create_vertex_buffer_layouts(
    shader: &crate::ShaderObject,
    mesh: &crate::mesh::MeshObject,
) -> Result<Option<VertexLayouts>, RendererError> {
    // Reflect inputs; if none, return None so callers can fall back to triangle path
    let vertex_inputs = shader.reflect_vertex_inputs()?;
    if vertex_inputs.is_empty() {
        return Ok(None);
    }

    // Read schemas
    let Some(vertex_schema) = mesh.vertex_schema.read().clone() else {
        return Err(RendererError::Error(
            "Mesh attribute mapping failed: no vertex schema".into(),
        ));
    };
    let instance_schema = mesh.instance_schema.read().clone();

    // Precompute name -> (offset, fmt) for both streams
    let (vertex_map, vertex_stride) = schema_offsets(&vertex_schema);
    let (instance_map, instance_stride) = if let Some(ref schema) = instance_schema {
        let (map, stride) = schema_offsets(schema);
        (Some(map), Some(stride))
    } else {
        (None, None)
    };

    // Partition attributes per stream based on mapping rule
    let mut vertex_attributes: Vec<wgpu::VertexAttribute> = Vec::new();
    let mut instance_attributes: Vec<wgpu::VertexAttribute> = Vec::new();

    // Build optional index->name maps from the first vertex/instance
    let (vertex_location, vertex_location_map) = mesh.first_vertex_location_map();
    let instance_location_map = mesh.first_instance_location_map();

    for vertex_input in vertex_inputs.iter() {
        let mut placed = false;

        // 1) Try instance by explicit index
        if let Some(name) = instance_location_map.get(&vertex_input.location)
            && let Some((offset, format_mesh)) =
                instance_map.as_ref().and_then(|map| map.get(name)).cloned()
        {
            if vertex_input.format != format_mesh {
                return Err(RendererError::Error(format!(
                    "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                    vertex_input.name, vertex_input.location, vertex_input.format, format_mesh
                )));
            }
            instance_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        // 2) Try vertex by explicit index (position or property)
        if !placed {
            // position index
            if vertex_input.location == vertex_location
                && let Some((offset, format_mesh)) = vertex_map.get("position").cloned()
            {
                if vertex_input.format != format_mesh {
                    return Err(RendererError::Error(format!(
                        "Type mismatch for vertex 'position' @location({}): shader expects {:?}, mesh has {:?}",
                        vertex_input.location, vertex_input.format, format_mesh
                    )));
                }
                vertex_attributes.push(wgpu::VertexAttribute {
                    format: vertex_input.format,
                    offset,
                    shader_location: vertex_input.location,
                });
                placed = true;
            }
            if !placed
                && let Some(name) = vertex_location_map.get(&vertex_input.location)
                && let Some((offset, format_mesh)) = vertex_map.get(name.as_str()).cloned()
            {
                if vertex_input.format != format_mesh {
                    return Err(RendererError::Error(format!(
                        "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                        vertex_input.name, vertex_input.location, vertex_input.format, format_mesh
                    )));
                }
                vertex_attributes.push(wgpu::VertexAttribute {
                    format: vertex_input.format,
                    offset,
                    shader_location: vertex_input.location,
                });
                placed = true;
            }
        }
        // 3) Fallback: match by name (instance then vertex)
        if !placed
            && let Some(ref mi) = instance_map
            && let Some((offset, format_mesh)) = mi.get(vertex_input.name.as_str()).cloned()
        {
            if vertex_input.format != format_mesh {
                return Err(RendererError::Error(format!(
                    "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                    vertex_input.name, vertex_input.location, vertex_input.format, format_mesh
                )));
            }
            instance_attributes.push(wgpu::VertexAttribute {
                format: vertex_input.format,
                offset,
                shader_location: vertex_input.location,
            });
            placed = true;
        }
        if !placed
            && let Some((offset, format_mesh)) = vertex_map.get(vertex_input.name.as_str()).cloned()
        {
            if vertex_input.format != format_mesh {
                return Err(RendererError::Error(format!(
                    "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                    vertex_input.name, vertex_input.location, vertex_input.format, format_mesh
                )));
            }
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

    // Build layouts; leak attributes to 'static for the pipeline builder
    vertex_attributes.sort_by_key(|attr| attr.shader_location);
    let vertex_layout = wgpu::VertexBufferLayout {
        array_stride: vertex_stride,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Box::leak(vertex_attributes.into_boxed_slice()),
    };

    let instance_attributes = match instance_attributes.is_empty() {
        true => None,
        false => Some(instance_attributes),
    };
    let instance_layout =
        if let (Some(attrs), Some(stride)) = (instance_attributes, instance_stride) {
            let mut instance_attributes = attrs;
            instance_attributes.sort_by_key(|a| a.shader_location);
            Some(wgpu::VertexBufferLayout {
                array_stride: stride,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: Box::leak(instance_attributes.into_boxed_slice()),
            })
        } else {
            None
        };

    Ok(Some((vertex_layout, instance_layout)))
}

fn create_bind_group_layouts(
    device: &wgpu::Device,
    shader: &crate::ShaderObject,
) -> HashMap<u32, wgpu::BindGroupLayout> {
    let uniforms = &shader.storage.read().uniforms;
    let mut group_entries: HashMap<u32, Vec<wgpu::BindGroupLayoutEntry>> = HashMap::new();
    for (_, size, uniform) in uniforms.values() {
        if uniform.name.contains('.') {
            continue;
        }
        // Push constants never occupy a bind group in native mode; skip here.
        if let UniformData::PushConstant(_) = &uniform.data {
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
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
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
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access,
                                format,
                                view_dimension,
                            },
                            count: None,
                        }
                    }
                    // NEW UPSTREAM API !!! (good news)
                    // @TODO figure out how this works; this is key for video textures from <video> elements
                    naga::ImageClass::External => wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension,
                            multisampled: false,
                        },
                        count: None,
                    },
                }
            }
            UniformData::Sampler(info) => wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(if info.comparison {
                    wgpu::SamplerBindingType::Comparison
                } else {
                    wgpu::SamplerBindingType::Filtering
                }),
                count: None,
            },
            UniformData::Storage(data) => {
                if let Some((_inner, span, access)) = data.first() {
                    let min = if *span == 0 { 16 } else { *span as u64 };
                    let min = unsafe { std::num::NonZeroU64::new_unchecked(min) };

                    wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: if access.is_readonly() {
                            wgpu::ShaderStages::VERTEX
                                | wgpu::ShaderStages::FRAGMENT
                                | wgpu::ShaderStages::COMPUTE
                        } else {
                            // Writable storage buffers are not allowed in VERTEX stage without special features
                            wgpu::ShaderStages::COMPUTE
                        },
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
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
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
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
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
    shader: &crate::ShaderObject,
    format: wgpu::TextureFormat,
    sample_count: u32,
    vertex_layouts: Option<VertexLayouts>,
    depth_format: Option<wgpu::TextureFormat>,
) -> RenderPipeline {
    let mut bind_group_layouts = create_bind_group_layouts(device, shader);

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

    // Determine push-constant roots
    let storage = shader.storage.read();
    let mut push_roots: Vec<(String, u32)> = Vec::new();
    for (name, (_ofs, _size, u)) in storage.uniforms.iter() {
        if name.contains('.') {
            continue;
        }
        if let UniformData::PushConstant(data) = &u.data
            && let Some((_inner, span)) = data.first()
        {
            push_roots.push((name.clone(), *span));
        }
    }
    drop(storage);

    #[cfg(wasm)]
    let fallback_required = true;

    #[cfg(not(wasm))]
    let fallback_required = {
        let mut fallback_required = false;

        if push_roots.len() > 1 {
            fallback_required = true;
        } else if let Some((_, sz)) = push_roots.first() {
            let lim = device.limits();
            if *sz > lim.max_push_constant_size {
                fallback_required = true;
            }
        }

        fallback_required
    };

    // Build shader module possibly rewriting push constants to uniforms in fallback mode
    let mut push_mode: Option<PushMode> = None;

    let shader_module = if push_roots.is_empty() || !fallback_required {
        if let Some((name, sz)) = push_roots.first() {
            push_mode = Some(PushMode::Native {
                root: name.clone(),
                size: *sz,
            });
        }
        let module = Cow::Owned(shader.module.clone());
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Naga(module),
        })
    } else {
        // Fallback push-constants mode: rewrite push constants to uniform buffers

        // Let's add a new new binding group id (beyond current ones)
        let mut max_group: u32 = 0;
        for (_, _, u) in shader.storage.read().uniforms.values() {
            if !u.name.contains('.') {
                max_group = max_group.max(u.group);
            }
        }
        let fallback_group = max_group + 1;

        // Assign bindings following the discovered push_roots order.
        // push_roots preserves the order we collected from storage.uniforms / naga globals.
        let mut module = shader.module.clone();

        // Map root name -> binding index according to push_roots sequence
        let mut ordered_map: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        for (i, (name, _span)) in push_roots.iter().enumerate() {
            ordered_map.insert(name.clone(), i as u32);
        }

        // Apply rewrite on globals matching push constant address space
        for (_handle, gv) in module.global_variables.iter_mut() {
            if matches!(gv.space, naga::AddressSpace::PushConstant)
                && let Some(name) = gv.name.clone()
                && let Some(binding) = ordered_map.get(&name)
            {
                gv.space = naga::AddressSpace::Uniform;
                gv.binding = Some(naga::ResourceBinding {
                    group: fallback_group,
                    binding: *binding,
                });
            }
        }
        let bindings_map = ordered_map;

        // Create layout for the fallback group with uniform buffers
        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        for (name, binding) in bindings_map.iter() {
            // Find span from storage metadata
            if let Some((_, span, _u)) = shader.storage.read().uniforms.get(name) {
                let min = {
                    let padded = wgpu::util::align_to(*span as u64, 16).max(16);
                    unsafe { std::num::NonZeroU64::new_unchecked(padded) }
                };
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(min),
                    },
                    count: None,
                });
            }
        }
        entries.sort_by_key(|e| e.binding);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout (fallback push)"),
            entries: &entries,
        });
        bind_group_layouts.insert(fallback_group, layout);
        push_mode = Some(PushMode::Fallback {
            group: fallback_group,
            bindings: bindings_map,
        });

        // Build module
        let module = Cow::Owned(module);
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader (fallback push->uniform)"),
            source: wgpu::ShaderSource::Naga(module),
        })
    };

    // Ensure placeholder empty layouts for missing lower-index groups so that the positional
    // indices of the pipeline layout match shader @group() numbers (important for fallback).
    if let Some(max_g) = bind_group_layouts.keys().max().copied() {
        for g in 0..=max_g {
            bind_group_layouts.entry(g).or_insert_with(|| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Empty Bind Group Layout (placeholder)"),
                    entries: &[],
                })
            });
        }
    }

    let mut sorted_groups: Vec<_> = bind_group_layouts.keys().collect();
    sorted_groups.sort();
    let mut bind_group_layouts_sorted: Vec<&wgpu::BindGroupLayout> = Vec::new();
    for g in sorted_groups.into_iter() {
        if let Some(l) = bind_group_layouts.get(g) {
            bind_group_layouts_sorted.push(l);
        }
    }

    // Pipeline layout with optional push-constant ranges
    let mut push_ranges: Vec<wgpu::PushConstantRange> = Vec::new();
    if let Some(PushMode::Native { root: _, size }) = &push_mode
        && *size > 0
    {
        push_ranges.push(wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            range: 0..*size,
        });
    }

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Default Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts_sorted,
        push_constant_ranges: &push_ranges,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    RenderPipeline {
        pipeline,
        bind_group_layouts: bind_group_layouts.clone(),
        push_mode,
    }
}

fn schema_offsets(
    schema: &crate::mesh::VertexSchema,
) -> (
    std::collections::HashMap<String, (u64, wgpu::VertexFormat)>,
    u64,
) {
    let mut map = std::collections::HashMap::new();
    let mut ofs = 0u64;
    for f in schema.fields.iter() {
        map.insert(f.name.clone(), (ofs, f.fmt));
        ofs += f.size;
    }
    (map, schema.stride)
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
    use std::thread;
    use std::time::Duration;

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

    // E2E: stress set() under contention while rendering to a texture target
    #[test]
    fn e2e_set_stress_during_render_last_wins() {
        pollster::block_on(async move {
            let r = Renderer::new();
            let target = r
                .create_texture_target([64u32, 64u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> {
  return vec4<f32>(0.2, 0.4, 0.6, 1.0);
}
            "#;
            let shader = crate::Shader::new(wgsl).expect("shader");

            // Writer thread: keep updating time; last-wins
            let shader_writer = shader.clone();
            let writer = thread::spawn(move || {
                for i in 0..1000 {
                    let t = i as f32 * 0.001;
                    let _ = shader_writer.set("time", t);
                    // Small pause to interleave with render
                    if i % 50 == 0 {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            });

            // Render loop; flushes pending automatically
            for _ in 0..1000u32 {
                // Also set resolution each frame; should never fail
                let _ = shader.set("resolution", [64.0f32, 64.0f32]);
                r.render(&shader, &target).expect("render");
            }

            writer.join().expect("writer thread");

            // Ensure any last enqueued writes (due to contention) are applied
            shader.object.flush_pending();

            // After the loop, we expect the last time value (approx 0.999)
            let last: f32 = shader.get("time").expect("time get");
            assert!(last > 0.95 && last <= 1.2, "unexpected last time: {}", last);
        });
    }

    #[test]
    fn sampler_comparison_binds_and_renders_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
@group(0) @binding(0) var shadowTex: texture_depth_2d;
@group(0) @binding(1) var shadowSamp: sampler_comparison;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,0.), vec2<f32>(1.,0.), vec2<f32>(0.,1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    let s = textureSampleCompare(shadowTex, shadowSamp, v.uv, 0.5);
    return vec4<f32>(s, s, s, 1.0);
}
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let depth = renderer
                .create_depth_texture([8u32, 8u32])
                .await
                .expect("depth");
            shader.set("shadowTex", &depth).expect("set depth");

            renderer.render(&shader, &target).expect("render ok");
        });
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
            let tex = renderer
                .create_texture_with_size(&pixels, [2u32, 2u32])
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
            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]),
                Vertex::new([0.5, -0.5, 0.0]),
                Vertex::new([0.0, 0.5, 0.0]),
            ]);
            pass.add_mesh(&mesh).expect("mesh is compatible");

            renderer.render(&pass, &target).expect("render ok");
        });
    }

    // Story: Render the same triangle twice using two instances with different offsets.
    #[test]
    fn render_with_mesh_two_instances() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("mesh", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]),
                Vertex::new([0.5, -0.5, 0.0]),
                Vertex::new([0.0, 0.5, 0.0]),
            ]);
            // Two instances with different offsets: provide an "offset" property to match the shader
            use crate::mesh::VertexValue;
            mesh.add_instance(
                Vertex::new([0.0, 0.0]).set("offset", VertexValue::F32x2([0.0, 0.0])),
            );
            mesh.add_instance(
                Vertex::new([0.25, 0.0]).set("offset", VertexValue::F32x2([0.25, 0.0])),
            );

            pass.add_mesh(&mesh).expect("mesh is compatible");
            renderer.render(&pass, &target).expect("render ok");
        });
    }

    // Story: Vertex-only mapping with vec2 position and uv attribute on vertices.
    #[test]
    fn ast_mapping_vertex_pos2_and_uv_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.uv = uv;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(v.uv, 0.0, 1.0); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("p", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5]).set("uv", [0.0, 0.0]),
                Vertex::new([0.5, -0.5]).set("uv", [1.0, 0.0]),
                Vertex::new([0.0, 0.5]).set("uv", [0.5, 1.0]),
            ]);
            pass.add_mesh(&mesh).expect("mesh is compatible");

            renderer.render(&pass, &target).expect("render ok");
        });
    }

    // Story: Vertex color attribute provided per-vertex; fragment returns color.
    #[test]
    fn ast_mapping_vertex_color_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) color: vec4<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  out.color = color;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("p", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]).set("color", [1.0, 0.0, 0.0, 1.0]),
                Vertex::new([0.5, -0.5, 0.0]).set("color", [0.0, 1.0, 0.0, 1.0]),
                Vertex::new([0.0, 0.5, 0.0]).set("color", [0.0, 0.0, 1.0, 1.0]),
            ]);
            pass.add_mesh(&mesh).expect("mesh is compatible");

            renderer.render(&pass, &target).expect("render ok");
        });
    }

    // Story: When a property name exists in both vertex and instance streams, instance is preferred.
    // We write per-instance "tint" and verify left/right instances draw with their tint colors.
    #[test]
    fn ast_mapping_instance_preferred_over_vertex() {
        fn read_pixel(img: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
            let i = ((y * w + x) * 4) as usize;
            [img[i], img[i + 1], img[i + 2], img[i + 3]]
        }

        pollster::block_on(async move {
            let renderer = Renderer::new();
            let size = [32u32, 32u32];
            let target = renderer
                .create_texture_target(size)
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) tint: vec4<f32> };
@vertex
fn vs_main(
  @location(0) pos: vec3<f32>,
  @location(1) offset: vec2<f32>,
  @location(2) tint: vec4<f32>
) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  out.tint = tint;
  return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> { return v.tint; }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("p", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::{Vertex, VertexValue};
            // Triangle geometry in clip-space
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0])
                    .set("tint", VertexValue::F32x4([0.0, 1.0, 0.0, 1.0])), // vertex-level green (should be ignored)
                Vertex::new([0.5, -0.5, 0.0]).set("tint", VertexValue::F32x4([0.0, 1.0, 0.0, 1.0])),
                Vertex::new([0.0, 0.5, 0.0]).set("tint", VertexValue::F32x4([0.0, 1.0, 0.0, 1.0])),
            ]);
            // Two instances with different offsets and tints
            mesh.add_instance(
                Vertex::new([-0.6, 0.0])
                    .set("offset", VertexValue::F32x2([-0.6, 0.0]))
                    .set("tint", VertexValue::F32x4([1.0, 0.0, 0.0, 1.0])), // red
            );
            mesh.add_instance(
                Vertex::new([0.6, 0.0])
                    .set("offset", VertexValue::F32x2([0.6, 0.0]))
                    .set("tint", VertexValue::F32x4([0.0, 0.0, 1.0, 1.0])), // blue
            );

            pass.add_mesh(&mesh).expect("mesh is compatible");
            renderer.render(&pass, &target).expect("render ok");

            let img = target.get_image();
            let w = size[0];
            // Helper: map NDC [-1,1] to pixel coordinate [0..w-1]
            let ndc_to_px = |x_ndc: f32, w: u32| -> u32 {
                let fx = (x_ndc * 0.5 + 0.5).clamp(0.0, 1.0);
                (fx * (w as f32 - 1.0)).round() as u32
            };
            let ndc_to_py = |y_ndc: f32, h: u32| -> u32 {
                // NDC +Y is up; pixel Y grows downward. Map accordingly.
                let fy = (-(y_ndc) * 0.5 + 0.5).clamp(0.0, 1.0);
                (fy * (h as f32 - 1.0)).round() as u32
            };
            let left_x = ndc_to_px(-0.6, size[0]);
            let right_x = ndc_to_px(0.6, size[0]);
            let y_mid = ndc_to_py(0.0, size[1]);

            let left = read_pixel(&img, w, left_x, y_mid);
            let right = read_pixel(&img, w, right_x, y_mid);

            // Expect roughly red on the left, blue on the right (alpha 255)
            // get_image() returns RGBA8 bytes
            assert!(
                left[0] > 128 && left[1] < 64 && left[2] < 64 && left[3] == 255,
                "left not red-ish (RGBA): {:?}",
                left
            );
            assert!(
                right[2] > 128 && right[1] < 64 && right[0] < 64 && right[3] == 255,
                "right not blue-ish (RGBA): {:?}",
                right
            );
        });
    }

    // Story: Shader with non-sequential location indices maps properly (uv at @location(3)).
    #[test]
    fn ast_mapping_reordered_locations_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(3) uv: vec2<f32>) -> VOut {
  var out: VOut;
  _ = uv; // not used in fragment
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.2,0.3,0.4,1.0); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("p", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]).set("uv", [0.0, 0.0]),
                Vertex::new([0.5, -0.5, 0.0]).set("uv", [1.0, 0.0]),
                Vertex::new([0.0, 0.5, 0.0]).set("uv", [0.5, 1.0]),
            ]);
            pass.add_mesh(&mesh).expect("mesh is compatible");

            renderer.render(&pass, &target).expect("render ok");
        });
    }

    // Story: "position" alias name (vec2) maps to mesh position2.
    #[test]
    fn ast_mapping_position_alias_position_name_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(position, 0.0, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.1,0.9,0.1,1.0); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("p", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5]),
                Vertex::new([0.5, -0.5]),
                Vertex::new([0.0, 0.5]),
            ]);
            pass.add_mesh(&mesh).expect("mesh is compatible");

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
    fn depth_msaa_sample_count_matches() {
        pollster::block_on(async move {
            let (adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            let color_format = wgpu::TextureFormat::Rgba8Unorm;
            let size = wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            };
            let frame = create_test_frame(&ctx.device, size, color_format);

            // Pick supported MSAA > 1 if available
            let flags = adapter.get_texture_format_features(color_format).flags;
            let sc = if flags.sample_count_supported(4) {
                4
            } else if flags.sample_count_supported(2) {
                2
            } else {
                1
            };

            if sc > 1 {
                ctx.set_sample_count(sc);
                // Create a matching-sample depth texture and register it
                let depth_obj = crate::TextureObject::create_depth_texture(&ctx, size);
                let depth_obj = std::sync::Arc::new(depth_obj);
                let depth_id = ctx.register_texture(depth_obj.clone());

                let shader = Shader::default();
                let pass = Pass::from_shader("msaa-depth", &shader);
                // Hold the passes slice to extend its lifetime, then clone the first Arc<PassObject>
                let first_pass = {
                    let passes = pass.passes();
                    passes.first().cloned().expect("pass")
                };
                first_pass.set_depth_target_id(depth_id);

                let mut encoder = ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let res = ctx.process_render_pass(&mut encoder, &first_pass, &frame, size);
                assert!(
                    res.is_ok(),
                    "msaa depth render should succeed: {:?}",
                    res.err()
                );
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

    // Story: AST-driven mapping errors when a needed attribute is missing.
    #[test]
    fn ast_mapping_missing_attribute_errors() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,0.,1.,1.); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("mesh", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]),
                Vertex::new([0.5, -0.5, 0.0]),
                Vertex::new([0.0, 0.5, 0.0]),
            ]);

            // Expect validation at call site: No instance property named "offset"
            let res = pass.add_mesh(&mesh);
            assert!(res.is_err());
            let s = format!("{}", res.unwrap_err());
            assert!(s.contains("No compatible shader exists for this mesh"));

            // User tries to render a Pass without a Mesh, a default quad is created
            let res = renderer.render(&pass, &target);
            assert!(res.is_ok());
        });
    }

    // Story: AST-driven mapping errors on type mismatch between shader input and mesh property format.
    #[test]
    fn ast_mapping_type_mismatch_errors() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,0.,1.); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("mesh", &shader);

            let mesh = crate::mesh::Mesh::new();
            use crate::mesh::{Vertex, VertexValue};
            mesh.add_vertices([
                Vertex::new([-0.5, -0.5, 0.0]),
                Vertex::new([0.5, -0.5, 0.0]),
                Vertex::new([0.0, 0.5, 0.0]),
            ]);
            // Add instance with wrong-typed "offset" (vec3 instead of vec2)
            mesh.add_instance(
                Vertex::new([0.0, 0.0]).set("offset", VertexValue::F32x3([0.0, 0.0, 0.0])),
            );

            // Expect validation at call site: type mismatch for "offset"
            let res = pass.add_mesh(&mesh);
            assert!(res.is_err());
            let s = format!("{}", res.unwrap_err());
            assert!(s.contains("No compatible shader exists for this mesh"));

            // User tries to render a Pass without a compatible Mesh, a default quad is created
            let res = renderer.render(&pass, &target);
            assert!(res.is_ok());
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

    // Story: Shader uses only group(1) uniform; renderer creates placeholder empty bind group for group 0.
    #[test]
    fn placeholder_empty_bind_group_for_lower_group() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
@group(1) @binding(0) var<uniform> Tint: vec4<f32>;
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    return out;
}
@fragment
fn main() -> @location(0) vec4<f32> {
    return Tint;
}
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            shader
                .set("Tint", [0.2f32, 0.4, 0.6, 1.0])
                .expect("set uniform");

            let res = renderer.render(&shader, &target);
            assert!(
                res.is_ok(),
                "render should succeed with placeholder group 0"
            );
        });
    }

    // Story: Depth sample_count mismatch vs pass sample_count returns a descriptive error.
    #[test]
    fn depth_msaa_sample_count_mismatch_errors() {
        pollster::block_on(async move {
            let (_adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            let color_format = wgpu::TextureFormat::Rgba8Unorm;
            let size = wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            };
            let frame = create_test_frame(&ctx.device, size, color_format);

            // Force pass sample_count = 4 while making depth texture sample_count = 1
            ctx.set_sample_count(4);
            let depth_obj = crate::TextureObject::create_depth_texture_with_count(&ctx, size, 1);
            let depth_obj = std::sync::Arc::new(depth_obj);
            let depth_id = ctx.register_texture(depth_obj.clone());

            let shader = Shader::default();
            let pass = Pass::from_shader("msaa-depth-mismatch", &shader);
            let first_pass = {
                let passes = pass.passes();
                passes.first().cloned().expect("pass")
            };
            first_pass.set_depth_target_id(depth_id);

            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            let res = ctx.process_render_pass(&mut encoder, &first_pass, &frame, size);
            assert!(res.is_err());
            let s = format!("{}", res.unwrap_err());
            assert!(s.contains("sample_count"), "unexpected error: {s}");
        });
    }

    // Story: Compute-only pass builds pipeline and dispatches with a simple uniform.
    #[test]
    fn compute_pipeline_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([4u32, 4u32])
                .await
                .expect("texture target");

            let wgsl = r#"
@group(0) @binding(0) var<uniform> U: vec4<f32>;
@compute @workgroup_size(1)
fn cs_main() { _ = U; }
            "#;
            let shader = crate::Shader::new(wgsl).expect("compute shader");
            let pass = Pass::from_shader("c", &shader);
            assert!(pass.is_compute(), "pass should be compute");

            // Set the uniform to ensure a buffer is uploaded
            shader
                .set("U", [1.0f32, 2.0, 3.0, 4.0])
                .expect("set uniform");

            // Render should only run compute path and succeed
            let res = renderer.render(&pass, &target);
            assert!(res.is_ok(), "compute render ok: {:?}", res.err());
        });
    }

    // Story: Per-pass color target override renders without error (offscreen color target).
    #[test]
    fn render_to_offscreen_color_target_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let present_target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("present target");
            let color_target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("offscreen color target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    return out;
}
@fragment
fn main() -> @location(0) vec4<f32> { return vec4<f32>(0.8, 0.2, 0.1, 1.0); }
            "#;
            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = Pass::from_shader("off", &shader);
            pass.add_target(&color_target).expect("add color target");

            let res = renderer.render(&pass, &present_target);
            assert!(
                res.is_ok(),
                "render with per-pass color target ok: {:?}",
                res.err()
            );
        });
    }
}
