use crate::{PassObject, ShaderHash, Target, TargetFrame, TextureData, TextureTarget, UniformData};
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
pub(crate) mod background;
mod external_texture;
mod unregister;

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
#[cfg_attr(mobile, derive(uniffi::Object))]
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

    /// Create a [Texture](crate::Texture) from a unified spec. The single
    /// entry point for every shape — bare bytes / path / URL / file (encoded),
    /// `(input, [w, h])` or `(input, Size)` for raw pixel bytes,
    /// `(input, TextureFormat)` for an explicit format override,
    /// `(input, TextureOptions)` for full control, or a `Mipmap`
    /// (built off the renderer thread) for a GPU-only upload.
    ///
    /// The CPU work (decode, mipmap chain, raw-byte wrap) runs on a background
    /// worker on every native target; the calling thread is never pinned.
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture(
        &self,
        spec: impl Into<crate::texture::TextureInput>,
    ) -> Result<crate::texture::Texture, RendererError> {
        let input = spec.into();

        // CloneOf and Empty don't produce a new TextureObject — handle them
        // here so the dispatcher in `TextureObject::from_input` only sees
        // variants it can actually upload.
        if let TextureData::CloneOf(tex) = input.data {
            return Ok(tex);
        }
        if matches!(input.data, TextureData::Empty) {
            return Err(RendererError::CreateTextureError(
                "create_texture requires source data; for an empty allocation use create_storage_texture(input)".into(),
            ));
        }

        let context = self.context(None).await?;
        let object = crate::TextureObject::from_input(context.clone(), input).await?;
        let object = std::sync::Arc::new(object);
        let id = context.register_texture(object.clone());
        Ok(crate::texture::Texture::new(context, object, id))
    }

    /// Create a storage-class texture from a [`crate::TextureInput`]. Same
    /// transport as `create_texture` — one vocabulary across the API. The
    /// `From<T>` impls cover the common shapes:
    ///
    /// - `(size, format)` → empty storage texture, no initial data.
    /// - `(size, format, bytes)` → storage texture pre-seeded with `bytes`.
    ///
    /// `options.usage` overrides the default storage usage mask
    /// (`STORAGE | TEXTURE | COPY_SRC | COPY_DST`); use
    /// `TextureOptions::with_usage(...)` for the typed builder. `options.size`
    /// is **required** for this entry — there's no source to infer dimensions
    /// from. Returns [`crate::texture::TextureError::InvalidInput`] when
    /// missing.
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture(
        &self,
        input: impl Into<crate::TextureInput>,
    ) -> Result<crate::texture::Texture, RendererError> {
        let crate::TextureInput { data, options } = input.into();
        let context = self.context(None).await?;
        let usage = options
            .usage
            .map(wgpu::TextureUsages::from_bits_truncate)
            .unwrap_or(
                wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
            );
        let size = options.size.ok_or_else(|| {
            RendererError::CreateTextureError(
                "create_storage_texture requires options.size — pass `(size, format)` or set the field explicitly".into(),
            )
        })?;
        let wgpu_size: wgpu::Extent3d = size.into();
        let wgpu_format: wgpu::TextureFormat = options.format.into();

        let bytes_opt: Option<Vec<u8>> = match data {
            crate::TextureData::Empty => None,
            crate::TextureData::Bytes(bytes) => Some(bytes),
            other => {
                return Err(RendererError::CreateTextureError(format!(
                    "create_storage_texture only accepts TextureData::Empty or Bytes for now (got {:?}); decode external sources first then pass the raw bytes",
                    std::mem::discriminant(&other)
                )));
            }
        };

        if let Some(ref bytes) = bytes_opt {
            if !usage.contains(wgpu::TextureUsages::COPY_DST) {
                return Err(RendererError::CreateTextureError(
                    "create_storage_texture with seed data requires COPY_DST in the usage mask"
                        .into(),
                ));
            }
            let bpp = crate::texture::bytes_per_pixel(wgpu_format);
            if bpp == 0 {
                return Err(RendererError::CreateTextureError(
                    "Unsupported format for create_storage_texture seed data (bytes-per-pixel is 0)"
                        .into(),
                ));
            }
            let expected = (wgpu_size.width as usize)
                .saturating_mul(wgpu_size.height as usize)
                .saturating_mul(wgpu_size.depth_or_array_layers.max(1) as usize)
                .saturating_mul(bpp as usize);
            if bytes.len() < expected {
                return Err(RendererError::CreateTextureError(format!(
                    "Seed data is {} bytes but the texture needs {}",
                    bytes.len(),
                    expected
                )));
            }
        }

        let obj = crate::TextureObject::new(
            context.as_ref(),
            wgpu_size,
            wgpu_format,
            usage,
            crate::texture::SamplerOptions::default(),
        )?;

        if let Some(bytes) = bytes_opt {
            let bpp = crate::texture::bytes_per_pixel(wgpu_format);
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &obj.inner,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpp * wgpu_size.width),
                    rows_per_image: Some(wgpu_size.height),
                },
                wgpu_size,
            );
        }
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
        let sample_count = context.sample_count();
        let obj = crate::TextureObject::create_depth_texture(
            context.as_ref(),
            size.into().into(),
            sample_count,
        );
        let obj = std::sync::Arc::new(obj);
        let id = context.register_texture(obj.clone());
        Ok(crate::texture::Texture::new(context, obj, id))
    }

    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture(
        &self,
        texture_id: crate::texture::TextureId,
    ) -> Result<(), RendererError> {
        unregister::unregister_texture(self, texture_id)
    }

    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub async fn read_texture(
        &self,
        texture_id: crate::texture::TextureId,
    ) -> Result<Vec<u8>, RendererError> {
        let context = self
            .context
            .read()
            .as_ref()
            .cloned()
            .ok_or(RendererError::NoContext)?;
        let texture = context
            .get_texture(&texture_id)
            .ok_or(RendererError::TextureNotFoundError(texture_id))?;
        Ok(crate::texture::read_pixels(&context, &texture).await?)
    }

    #[cfg(wasm)]
    #[lsp_doc("docs/api/web/hidden/external_texture.md")]
    pub fn create_external_texture(
        &self,
        video: &web_sys::HtmlVideoElement,
    ) -> Result<external_texture::ExternalTextureHandle, RendererError> {
        external_texture::ExternalTextureHandle::from_video(self, video)
    }

    /// Realize every pending GPU upload referenced by `renderable` — the
    /// texture inputs queued by Material's lazy `*_texture` setters, the
    /// loader-built Scenes, and so on. After `load` returns, `render` against
    /// the same renderable is GPU-only.
    ///
    /// `render` runs `load` automatically the first time it sees a renderable
    /// with pending uploads, so calling `load` is optional; reach for it when
    /// you want to amortize the decode + upload cost outside the render loop.
    #[lsp_doc("docs/api/core/renderer/load.md")]
    pub async fn load(&self, renderable: &impl Renderable) -> Result<(), RendererError> {
        for pass in renderable.passes().iter() {
            let shaders: Vec<Arc<crate::shader::ShaderObject>> =
                pass.shaders.read().iter().cloned().collect();
            for shader in shaders {
                let pending = shader.drain_pending_textures();
                if pending.is_empty() {
                    continue;
                }
                for entry in pending {
                    let texture = self.create_texture(entry.input).await?;
                    let meta = crate::texture::TextureMeta::with_id_only(*texture.id());
                    let _ = shader.set(&entry.key, UniformData::Texture(meta));
                }
            }
        }
        Ok(())
    }

    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        // Drain any pending texture uploads from lazy Material setters before
        // we touch the GPU. Cheap when nothing is pending — the load future
        // doesn't await anything in that case. On native we block via
        // `futures::executor`; on wasm there's no blocking executor, so the
        // wasm async wrapper (`render_js`) is responsible for awaiting
        // `load` explicitly.
        #[cfg(not(wasm))]
        futures::executor::block_on(self.load(renderable))?;
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
        self.configure_surface(surface, size).await
    }

    /// Like `create_surface`, but takes an already-constructed `wgpu::Surface`
    /// (usually one built from a raw pointer via `instance.create_surface_unsafe`).
    /// Callers are responsible for ensuring the backing window/layer outlives the surface.
    #[cfg(ios)]
    pub(crate) async fn configure_unsafe_surface(
        &self,
        target: wgpu::SurfaceTargetUnsafe,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let instance = self.instance().await;
        // SAFETY: the caller promises the underlying handle (CAMetalLayer / ANativeWindow / ...)
        // remains valid for the lifetime of the returned Surface.
        let surface = unsafe { instance.create_surface_unsafe(target)? };
        self.configure_surface(surface, size).await
    }

    async fn configure_surface<'window>(
        &self,
        surface: wgpu::Surface<'window>,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'window>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
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

/// One translucent draw to issue inside the blend phase of a pass. The
/// renderer builds these from `Pass::model_entries` whose Material declares
/// `alpha_mode: Blend`, sorts them back-to-front by `eye_z`, then walks the
/// sorted list reusing each shader's already-built pipeline + bind groups
/// with a fresh single-instance vertex buffer per draw. Per-entry buffers
/// kill the instance-batching for blend draws (which is unavoidable when
/// over-blending needs strict depth ordering) but every other allocation
/// stays cached across draws.
#[derive(Debug)]
struct BlendDraw {
    /// Pointer identity of the shader Arc. Same key the opaque batching
    /// uses, looked up by the blend-phase loop to find the cached
    /// pipeline + bind groups (filed under this key in `process_render_pass`).
    shader_ptr: usize,
    /// Pointer identity of the mesh Arc. Pairs with `shader_ptr` for the
    /// vertex-buffer lookup.
    #[allow(dead_code)]
    mesh_ptr: usize,
    /// Strong handle to the shader, kept here so `shader_ptr` stays valid
    /// throughout the render pass — the ModelEntry's Arc release at end
    /// of `build_pass_draws` would otherwise leave the raw pointer
    /// dangling against a freed allocation.
    #[allow(dead_code)]
    shader: Arc<crate::shader::ShaderObject>,
    /// Strong handle to the mesh so we can fetch its vertex + index buffers
    /// when the draw runs.
    mesh: Arc<crate::mesh::MeshObject>,
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
    instance_buffer: Option<wgpu::Buffer>,
}

/// Output of `Renderer::build_pass_draws`. Splits the pass's Model entries
/// into the two draw-time categories the renderer cares about: batched
/// opaque/mask draws (one instanced GPU call per `(shader, mesh)` pair)
/// and individually-ordered blend draws.
#[derive(Default, Debug)]
struct PassDraws {
    /// Per-(shader_ptr, mesh_ptr) instance buffer + instance count. Keys
    /// match the `Pass::add_model` dedupe identity. Bound at vertex slot 1
    /// for the corresponding shader+mesh draw.
    opaque_overrides: HashMap<(usize, usize), (wgpu::Buffer, u32)>,
    /// Translucent draws, **sorted back-to-front** by eye-space Z. The
    /// renderer walks this list inside the blend phase, looking each
    /// shader's pipeline + bind groups up by `shader_ptr`.
    blend_draws: Vec<BlendDraw>,
}

/// Key for caching render pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RenderPipelineKey {
    shader_hash: ShaderHash,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    sample_count: u32,
    /// `Material::alpha_mode` bakes into the pipeline: `Opaque`/`Mask`
    /// turn blending off, `Blend` flips on standard alpha-over and disables
    /// depth-write. Different modes against the same shader hash therefore
    /// need different pipelines.
    alpha_mode: crate::material::AlphaMode,
    /// `Material::double_sided` flips the `cull_mode` on the primitive
    /// state, so it also baked into the pipeline.
    double_sided: bool,
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

    // Lazy-init 1x1 fallback textures for the default PBR shader's glTF
    // texture-map bindings. Populated synchronously on first use inside
    // `process_render_pass`, so `Material::pbr` doesn't need a Renderer.
    pbr_defaults: RwLock<Option<Arc<PbrDefaults>>>,
}

#[derive(Debug)]
pub(crate) struct PbrDefaults {
    /// Five (view, sampler) pairs the renderer binds to the PBR-named slots
    /// when a Material::pbr Shader doesn't have a user-supplied texture for
    /// the slot. The byte payload is chosen so `factor * sample = factor`
    /// (binding the default is equivalent to "no map").
    pub(crate) base_color: (wgpu::TextureView, wgpu::Sampler),
    pub(crate) metallic_roughness: (wgpu::TextureView, wgpu::Sampler),
    pub(crate) normal: (wgpu::TextureView, wgpu::Sampler),
    pub(crate) occlusion: (wgpu::TextureView, wgpu::Sampler),
    pub(crate) emissive: (wgpu::TextureView, wgpu::Sampler),
}

impl PbrDefaults {
    fn slot(&self, uniform_name: &str) -> Option<&(wgpu::TextureView, wgpu::Sampler)> {
        match uniform_name {
            "base_color_map" => Some(&self.base_color),
            "metallic_roughness_map" => Some(&self.metallic_roughness),
            "normal_map" => Some(&self.normal),
            "occlusion_map" => Some(&self.occlusion),
            "emissive_map" => Some(&self.emissive),
            _ => None,
        }
    }
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
            pbr_defaults: RwLock::new(None),
        }
    }

    /// Borrow the lazy 1x1 PBR fallback textures, creating them on first use.
    /// Used to fill the default PBR shader's texture-map bindings when a
    /// `Material::pbr` doesn't supply its own texture for the slot. Pure
    /// sync calls against the wgpu device — no FC texture registry, no
    /// async hop.
    fn pbr_defaults(&self) -> Arc<PbrDefaults> {
        if let Some(existing) = self.pbr_defaults.read().clone() {
            return existing;
        }
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PBR default sampler"),
            ..Default::default()
        });
        let make = |label: &'static str, rgba: [u8; 4]| -> wgpu::TextureView {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            tex.create_view(&wgpu::TextureViewDescriptor::default())
        };
        // Defaults chosen so `factor * sample = factor` in the shader.
        let bundle = Arc::new(PbrDefaults {
            base_color: (make("PBR default base_color", [255, 255, 255, 255]), sampler.clone()),
            // glTF stores metallic in .b and roughness in .g; the shader's
            // .bgr swizzle puts metallic in .r (=1) and roughness in .g (=1).
            metallic_roughness: (make("PBR default metallic_roughness", [0, 255, 255, 255]), sampler.clone()),
            // Neutral tangent-space normal (0, 0, 1) after the * 2.0 - 1.0 decode.
            normal: (make("PBR default normal", [128, 128, 255, 255]), sampler.clone()),
            occlusion: (make("PBR default occlusion", [255, 255, 255, 255]), sampler.clone()),
            emissive: (make("PBR default emissive", [255, 255, 255, 255]), sampler),
        });
        *self.pbr_defaults.write() = Some(bundle.clone());
        bundle
    }

    /// Renders any `Renderable` (Shader, Pass, or iterable of Pass) to a Target.
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

        let frame = self.acquire_frame(target)?;

        let pass_list = renderable.passes();
        // Apple's Metal driver does not reliably flush tile-based storage-texture writes between
        // two compute passes recorded in the same command buffer, so a subsequent sampled
        // (`texture_2d<…>`) read returns zeros on Apple Silicon. Submitting the encoder between
        // sequential compute passes introduces a submission boundary that reliably flushes the
        // tile memory. Render passes are unaffected (encoder-end already syncs there).
        #[cfg(apple)]
        let mut prev_was_compute = false;
        for pass in pass_list.iter() {
            let pass = pass.as_ref();
            #[cfg(apple)]
            {
                if prev_was_compute && pass.is_compute() {
                    self.queue.submit(Some(encoder.finish()));
                    encoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Command Encoder (Metal compute sync split)"),
                        });
                }
                prev_was_compute = pass.is_compute();
            }
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
    fn acquire_frame(
        &self,
        target: &impl Target,
    ) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        match target.get_current_frame() {
            Ok(f) => Ok(f),
            Err(crate::SurfaceError::Lost) | Err(crate::SurfaceError::Outdated) => {
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
        let texture_id = crate::texture::TextureId { id };
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

    /// Run a wgpu operation inside a validation error scope so that any
    /// validation failure surfaces as a typed
    /// `RendererError::ValidationError` instead of being swallowed by the
    /// device's `on_uncaptured_error` handler. Wrap any wgpu call whose
    /// failure mode would otherwise leak to stderr — `create_bind_group`,
    /// `Texture::create_view`, etc.
    ///
    /// On wasm, `pop_error_scope` resolves asynchronously and cannot be
    /// awaited from this sync render path; the closure runs and its
    /// result is returned unwrapped, matching the prior behavior.
    fn validate<T>(&self, label: &str, op: impl FnOnce() -> T) -> Result<T, RendererError> {
        #[cfg(not(wasm))]
        {
            let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
            let value = op();
            if let Some(err) = futures::executor::block_on(scope.pop()) {
                return Err(RendererError::ValidationError {
                    label: label.to_string(),
                    message: err.to_string(),
                });
            }
            Ok(value)
        }
        #[cfg(wasm)]
        {
            let _ = label;
            Ok(op())
        }
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
    fn build_pass_draws(&self, pass: &PassObject) -> PassDraws {
        let entries = pass.model_entries.read();
        let mut out = PassDraws::default();
        if entries.is_empty() {
            return out;
        }

        // Compute eye-Z via the cached view matrix; `None` means no Camera
        // was attached to the pass, in which case we leave `eye_z` at 0
        // and the sort is a no-op (stable, insertion order preserved).
        let view_mat: Option<glam::Mat4> = pass
            .camera_snapshot
            .read()
            .as_ref()
            .map(|snap| glam::Mat4::from_cols_array_2d(&snap.view));

        let mut opaque_groups: HashMap<(usize, usize), Vec<u8>> = HashMap::new();
        for entry in entries.iter() {
            // `Model::set_visible(false)` removes the entry from this frame's
            // draw queue. Checked here so it cuts both the opaque-batched
            // and blend-sorted paths in one place (no instance-buffer row
            // packed, no BlendDraw queued, no eye-Z computed).
            if !*entry.visible.read() {
                continue;
            }
            let alpha_mode = *entry.shader.alpha_mode.read();
            let key = (
                Arc::as_ptr(&entry.shader) as usize,
                Arc::as_ptr(&entry.mesh) as usize,
            );
            let transform = *entry.transform.read();
            match alpha_mode {
                crate::material::AlphaMode::Opaque | crate::material::AlphaMode::Mask => {
                    let m = transform.to_cols_array_2d();
                    let slot = opaque_groups.entry(key).or_default();
                    for col in m.iter() {
                        for v in col.iter() {
                            slot.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                }
                crate::material::AlphaMode::Blend => {
                    // Eye-space Z of the model's origin (its world position is
                    // `transform * (0, 0, 0, 1)` → the fourth column). Multiplied
                    // by the view matrix to land in camera space; the `.z`
                    // component is the depth we sort against.
                    //
                    // Why model origin and not bounding-box centroid? Centroid
                    // would be more accurate for elongated meshes but requires
                    // mesh AABB plumbing we don't have yet. Origin works
                    // for the common case (mesh centered around its local
                    // origin); if it ever falls short the fix is to plumb
                    // per-Mesh world-space AABB through `MeshObject`.
                    let origin_world = transform * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
                    let eye_z = view_mat
                        .map(|v| (v * origin_world).z)
                        .unwrap_or(0.0);
                    out.blend_draws.push(BlendDraw {
                        shader_ptr: key.0,
                        mesh_ptr: key.1,
                        shader: entry.shader.clone(),
                        mesh: entry.mesh.clone(),
                        transform,
                        eye_z,
                        instance_buffer: None,
                    });
                }
            }
        }

        for (key, bytes) in opaque_groups {
            let count = (bytes.len() / 64) as u32;
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pass model instance buffer"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&buffer, 0, &bytes);
            out.opaque_overrides.insert(key, (buffer, count));
        }

        // Back-to-front: highest `eye_z` is closest to the eye in
        // right-handed view space (the camera looks down its -Z axis, so
        // farther geometry has a *more negative* Z and a *smaller* `eye_z`
        // value). We want to draw the smallest values first.
        out.blend_draws
            .sort_by(|a, b| a.eye_z.partial_cmp(&b.eye_z).unwrap_or(std::cmp::Ordering::Equal));

        // Materialize one tiny instance buffer per blend draw — 64 bytes
        // each. Cheap to allocate, cheaper to recycle into a buffer pool
        // later if profiling shows a need.
        for draw in out.blend_draws.iter_mut() {
            let m = draw.transform.to_cols_array_2d();
            let mut bytes = Vec::with_capacity(64);
            for col in m.iter() {
                for v in col.iter() {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Pass blend model instance buffer"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&buffer, 0, &bytes);
            draw.instance_buffer = Some(buffer);
        }

        out
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

        // Snapshot per-Model transforms for this Pass into GPU instance
        // buffers BEFORE begin_render_pass — the wgpu::Buffer handles need
        // to outlive the render_pass. Opaque/Mask entries collapse to one
        // batched buffer per `(shader_ptr, mesh_ptr)` (auto-instancing);
        // Blend entries get one buffer each, kept in a sorted Vec so the
        // blend phase can walk them back-to-front by eye-space depth.
        let pass_draws = self.build_pass_draws(pass);
        let model_overrides = &pass_draws.opaque_overrides;
        let blend_draws = &pass_draws.blend_draws;

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
            multiview_mask: None,
        };
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        let required_size = *pass.required_buffer_size.read();
        self.buffer_pool
            .write()
            .ensure_capacity(required_size, &self.device);

        for shader in pass.shaders.read().iter() {
            shader.flush_pending();
            let shader_meshes = shader.meshes.read().clone();
            let layouts = if let Some(first_mesh) = shader_meshes.first() {
                // Ensure schemas are derived (and buffers packed) before pipeline creation
                first_mesh.vertex_buffers(&self.device, &self.queue)?;
                // Now derive vertex buffer layouts; propagate any mapping error
                vertex_buffer_layouts(shader, first_mesh.as_ref())?
            } else {
                None
            };

            // Snapshot the Material-owned pipeline-state flags from the
            // shader's back-references. Drop the read locks immediately so
            // we don't hold them across the pipeline build.
            let alpha_mode = *shader.alpha_mode.read();
            let double_sided = *shader.double_sided.read();
            let pipeline_key = RenderPipelineKey {
                shader_hash: shader.hash,
                color_format,
                depth_format,
                sample_count,
                alpha_mode,
                double_sided,
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
                        layouts,
                        depth_format,
                        alpha_mode,
                        double_sided,
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
            let mut pbr_defaults_handle: Option<Arc<PbrDefaults>> = None;
            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                match &uniform.data {
                    UniformData::Texture(meta) => {
                        if let Some(tex) = self.get_texture(&meta.id) {
                            let view = self.validate(
                                &format!("texture view for binding '{}'", name),
                                || tex.create_view(),
                            )?;
                            let sampler = tex.sampler();
                            let group_entry = groups.entry(uniform.group).or_default();
                            group_entry.views.push((uniform.binding, view));
                            group_entry.last_texture_sampler = Some(sampler);
                        } else {
                            // No user texture — fall back to the PBR default for
                            // the known glTF slot names so `Material::pbr` works
                            // without the caller supplying every map.
                            let defaults = pbr_defaults_handle
                                .get_or_insert_with(|| self.pbr_defaults());
                            if let Some((view, sampler)) = defaults.slot(name) {
                                let group_entry = groups.entry(uniform.group).or_default();
                                group_entry.views.push((uniform.binding, view.clone()));
                                group_entry.last_texture_sampler = Some(sampler.clone());
                            } else {
                                log::warn!(
                                    "Texture handle {:?} not found for uniform {}",
                                    meta.id,
                                    name
                                );
                            }
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
                                mipmap_filter: wgpu::MipmapFilterMode::Linear,
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
                        if let Some(crate::shader::uniform::StorageEntry { span, .. }) =
                            data.first()
                        {
                            // Acquire persistent storage buffer and upload only if necessary
                            let span_u64 = *span as u64;
                            // Obtain initial bytes for creation or update.
                            // Blocking read: waits briefly (microseconds) if a `set` is
                            // mid-write. `try_read` here would surface as `Busy`
                            // mid-render under stress (writer thread holds the
                            // write lock for the duration of one `update` call).
                            // No deadlock risk — render doesn't re-enter `set` on
                            // the same thread.
                            let init_bytes: Vec<u8> = {
                                let s = shader.storage.read();
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
                            // If CPU blob is marked dirty, upload and clear flag.
                            // Blocking acquisitions (read for the snapshot, write
                            // for the dirty-flag clear) — see comment on the
                            // earlier `init_bytes` block for the rationale.
                            let need_upload = {
                                let s = shader.storage.read();
                                s.is_storage_dirty(name)
                            };
                            if need_upload {
                                let bytes: Vec<u8> = {
                                    let s = shader.storage.read();
                                    s.get_bytes(name)
                                        .map(|b| b.to_vec())
                                        .unwrap_or_else(|| vec![0u8; span_u64 as usize])
                                };
                                self.queue.write_buffer(&buf, 0, &bytes);
                                let mut s = shader.storage.write();
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

                let label = format!("Bind Group for group: {}", group_index);
                let bind_group = self.validate(&label, || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &entries,
                        label: Some(&label),
                    })
                })?;
                present_groups.insert(group_index);
                bind_groups.push((group_index, bind_group));
            }

            // Ensure empty bind groups are set for placeholder layouts the pipeline expects
            for (group, layout) in cached_pipeline.bind_group_layouts.iter() {
                if !present_groups.contains(group) {
                    // Create an empty bind group (layout is expected to have zero entries for placeholders)
                    let label = format!("Empty Bind Group for group: {}", group);
                    let empty = self.validate(&label, || {
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout,
                            entries: &[],
                            label: Some(&label),
                        })
                    })?;
                    bind_groups.push((*group, empty));
                }
            }

            // Sort by group index to match pipeline layout order
            bind_groups.sort_by_key(|(g, _)| *g);
            render_pass.set_pipeline(&cached_pipeline.pipeline);
            for (group, bind_group) in bind_groups.iter() {
                render_pass.set_bind_group(*group, bind_group, &[]);
            }

            // Set native immediate data just before draw if applicable.
            // Blocking read for the same reason as the storage-buffer snapshot
            // earlier in this function.
            if let Some(PushMode::Native { root, .. }) = &cached_pipeline.push_mode {
                let storage = shader.storage.read();
                if let Some(bytes) = storage.get_bytes(root) {
                    render_pass.set_immediates(0, bytes);
                }
            }

            match shader_meshes.len() {
                0 => render_pass.draw(0..3, 0..1),
                _ => {
                    let shader_ptr = Arc::as_ptr(shader) as usize;
                    match alpha_mode {
                        crate::material::AlphaMode::Opaque
                        | crate::material::AlphaMode::Mask => {
                            // Auto-instancing path: one batched draw per
                            // (shader, mesh) pair regardless of how many
                            // Models contributed. The instance override map
                            // packs all queued model matrices into one
                            // vertex buffer at slot 1.
                            for mesh_object in shader_meshes.iter() {
                                let (refs, counts) =
                                    mesh_object.vertex_buffers(&self.device, &self.queue)?;
                                render_pass.set_vertex_buffer(0, refs.vertex_buffer.slice(..));

                                // Per-Pass Model override takes precedence over the
                                // Mesh's own instance buffer. The override carries N
                                // model matrices (one per Model::add_model call that
                                // referenced this shader+mesh); the fall-through path
                                // is for Meshes whose instances came from a direct
                                // `Mesh::add_instance` call (crowd rendering, etc.).
                                let override_key =
                                    (shader_ptr, Arc::as_ptr(mesh_object) as usize);
                                let instance_count = if let Some((buf, count)) =
                                    model_overrides.get(&override_key)
                                {
                                    render_pass.set_vertex_buffer(1, buf.slice(..));
                                    *count
                                } else {
                                    if let Some(instance_buffer) = &refs.instance_buffer {
                                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                    }
                                    counts.instance_count
                                };

                                render_pass.set_index_buffer(
                                    refs.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                render_pass.draw_indexed(
                                    0..counts.index_count,
                                    0,
                                    0..instance_count,
                                );
                            }
                        }
                        crate::material::AlphaMode::Blend => {
                            // Translucent path: instance batching is unsafe
                            // because two overlapping translucent fragments
                            // need strict back-to-front draw order for the
                            // over-blend to converge to the right color.
                            // `blend_draws` was sorted by eye-space Z when
                            // we built it (far→near). Walk the subset for
                            // this shader in that order and submit one
                            // draw per entry with its own single-instance
                            // vertex buffer at slot 1.
                            //
                            // Known limitation: this is per-shader sorting.
                            // Multiple translucent Materials in the same
                            // Pass interleave per-shader, not globally —
                            // shader A's farthest is drawn after shader B's
                            // nearest if B comes earlier in `pass.shaders`.
                            // In practice most scenes use one translucent
                            // Material (shared shader); cross-Material
                            // interleaving rides on a follow-up.
                            for draw in blend_draws.iter() {
                                if draw.shader_ptr != shader_ptr {
                                    continue;
                                }
                                let Some(buf) = draw.instance_buffer.as_ref() else {
                                    continue;
                                };
                                let (refs, counts) =
                                    draw.mesh.vertex_buffers(&self.device, &self.queue)?;
                                render_pass.set_vertex_buffer(0, refs.vertex_buffer.slice(..));
                                render_pass.set_vertex_buffer(1, buf.slice(..));
                                render_pass.set_index_buffer(
                                    refs.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                render_pass.draw_indexed(0..counts.index_count, 0, 0..1);
                            }
                        }
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
            let layouts = bind_group_layouts(&self.device, shader);

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
                    let mut sorted_layouts: Vec<Option<&wgpu::BindGroupLayout>> = Vec::new();
                    for g in sorted_groups.into_iter() {
                        sorted_layouts.push(layouts.get(&g));
                    }

                    let layout =
                        self.device
                            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: Some("Compute Pipeline Layout"),
                                bind_group_layouts: &sorted_layouts,
                                immediate_size: 0,
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
                            let view = self.validate(
                                &format!("texture view for binding '{}'", name),
                                || tex.create_view(),
                            )?;
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
                                mipmap_filter: wgpu::MipmapFilterMode::Linear,
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
                        if let Some(crate::shader::uniform::StorageEntry {
                            span: span_u32, ..
                        }) = data.first()
                        {
                            let span = *span_u32 as u64;
                            // Obtain bytes. Blocking read — see comment on the
                            // first storage-buffer init block above.
                            let init_bytes: Vec<u8> = {
                                let s = shader.storage.read();
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
                            // Upload if dirty. Blocking acquisitions — see
                            // comment on the earlier dirty-flag block above.
                            let need_upload = {
                                let s = shader.storage.read();
                                s.is_storage_dirty(name)
                            };
                            if need_upload {
                                let bytes: Vec<u8> = {
                                    let s = shader.storage.read();
                                    s.get_bytes(name)
                                        .map(|b| b.to_vec())
                                        .unwrap_or_else(|| vec![0u8; span as usize])
                                };
                                self.queue.write_buffer(&buf, 0, &bytes);
                                let mut s = shader.storage.write();
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
                        // Blocking read — same rationale as the storage-buffer
                        // path above.
                        let storage = shader.storage.read();
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
                let label = format!("Compute Bind Group for group: {}", group);
                let bind_group = self.validate(&label, || {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &entries,
                        label: Some(&label),
                    })
                })?;
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
fn vertex_buffer_layouts(
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
            && let Some((offset, format_mesh)) = vertex_map.get("position").cloned()
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

fn bind_group_layouts(
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
                use crate::texture::{
                    TextureClass, TextureDim, TextureScalarKind, TextureStorageFormat,
                };
                // Map our uniffi-friendly metadata to wgpu binding types
                let view_dimension = match (meta.dim, meta.arrayed) {
                    (TextureDim::D2, false) => wgpu::TextureViewDimension::D2,
                    (TextureDim::D2, true) => wgpu::TextureViewDimension::D2Array,
                    (TextureDim::D3, _) => wgpu::TextureViewDimension::D3,
                    (TextureDim::Cube, false) => wgpu::TextureViewDimension::Cube,
                    (TextureDim::Cube, true) => wgpu::TextureViewDimension::CubeArray,
                    _ => wgpu::TextureViewDimension::D2,
                };
                match &meta.class {
                    TextureClass::Depth { .. } => wgpu::BindGroupLayoutEntry {
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
                    TextureClass::Sampled { kind, multi } => {
                        let sample_type = match kind {
                            TextureScalarKind::Sint => wgpu::TextureSampleType::Sint,
                            TextureScalarKind::Uint => wgpu::TextureSampleType::Uint,
                            _ => wgpu::TextureSampleType::Float {
                                filterable: meta.sampled,
                            },
                        };
                        wgpu::BindGroupLayoutEntry {
                            binding: uniform.binding,
                            visibility: wgpu::ShaderStages::VERTEX
                                | wgpu::ShaderStages::FRAGMENT
                                | wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type,
                                view_dimension,
                                multisampled: *multi,
                            },
                            count: None,
                        }
                    }
                    TextureClass::Storage { format, access } => {
                        let access = if access.load && access.store {
                            wgpu::StorageTextureAccess::ReadWrite
                        } else if access.load {
                            wgpu::StorageTextureAccess::ReadOnly
                        } else if access.store {
                            wgpu::StorageTextureAccess::WriteOnly
                        } else {
                            wgpu::StorageTextureAccess::ReadOnly
                        };
                        let format = match format {
                            TextureStorageFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
                            TextureStorageFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
                            TextureStorageFormat::R8Uint => wgpu::TextureFormat::R8Uint,
                            TextureStorageFormat::R8Sint => wgpu::TextureFormat::R8Sint,
                            TextureStorageFormat::R16Uint => wgpu::TextureFormat::R16Uint,
                            TextureStorageFormat::R16Sint => wgpu::TextureFormat::R16Sint,
                            TextureStorageFormat::R16Float => wgpu::TextureFormat::R16Float,
                            TextureStorageFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
                            TextureStorageFormat::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
                            TextureStorageFormat::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
                            TextureStorageFormat::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
                            TextureStorageFormat::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
                            TextureStorageFormat::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
                            TextureStorageFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
                            TextureStorageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
                            TextureStorageFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
                            TextureStorageFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
                            TextureStorageFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
                            TextureStorageFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
                            TextureStorageFormat::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
                            TextureStorageFormat::Rg11b10Ufloat => {
                                wgpu::TextureFormat::Rg11b10Ufloat
                            }
                            TextureStorageFormat::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
                            TextureStorageFormat::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
                            TextureStorageFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
                            TextureStorageFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
                            TextureStorageFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
                            TextureStorageFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
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
                    // External textures (Web): bind as ExternalTexture when available.
                    TextureClass::External() => wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::ExternalTexture,
                        count: None,
                    },
                }
            }
            UniformData::Sampler(info) => wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(if info.comparison {
                    wgpu::SamplerBindingType::Comparison
                } else {
                    wgpu::SamplerBindingType::Filtering
                }),
                count: None,
            },
            UniformData::Storage(data) => {
                if let Some(crate::shader::uniform::StorageEntry { span, access, .. }) =
                    data.first()
                {
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
    alpha_mode: crate::material::AlphaMode,
    double_sided: bool,
) -> RenderPipeline {
    let mut layouts = bind_group_layouts(device, shader);

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
            && let Some(crate::shader::uniform::PushEntry { span, .. }) = data.first()
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
            if *sz > lim.max_immediate_size {
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

        // Apply rewrite on globals matching immediate/push-constant address space
        for (_handle, gv) in module.global_variables.iter_mut() {
            if matches!(gv.space, naga::AddressSpace::Immediate)
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
        layouts.insert(fallback_group, layout);
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
    if let Some(max_g) = layouts.keys().max().copied() {
        for g in 0..=max_g {
            layouts.entry(g).or_insert_with(|| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Empty Bind Group Layout (placeholder)"),
                    entries: &[],
                })
            });
        }
    }

    let mut sorted_groups: Vec<_> = layouts.keys().collect();
    sorted_groups.sort();
    let mut sorted_layouts: Vec<Option<&wgpu::BindGroupLayout>> = Vec::new();
    for g in sorted_groups.into_iter() {
        sorted_layouts.push(layouts.get(g));
    }

    // Pipeline layout with optional immediate (push-constant) size
    let immediate_size = match &push_mode {
        Some(PushMode::Native { size, .. }) => *size,
        _ => 0,
    };

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Default Pipeline Layout"),
        bind_group_layouts: &sorted_layouts,
        immediate_size,
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
                // Blend state baked from Material::alpha_mode:
                //   Opaque / Mask -> no blending (alpha bits go to the FB
                //                    but the equation ignores them; Mask
                //                    does its work via `discard` in the
                //                    fragment shader).
                //   Blend         -> standard SrcAlpha/OneMinusSrcAlpha
                //                    over-blend.
                blend: match alpha_mode {
                    crate::material::AlphaMode::Opaque
                    | crate::material::AlphaMode::Mask => None,
                    crate::material::AlphaMode::Blend => {
                        Some(wgpu::BlendState::ALPHA_BLENDING)
                    }
                },
                // Custom (non-glTF-MR) blend equations are tracked under
                // the roadmap's "Custom blending" item — Material's
                // AlphaMode covers the three glTF 2.0 modes; anything more
                // exotic will ride on a separate per-Material slot.
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            // `double_sided=true` (the ShaderObject default) leaves
            // `cull_mode: None`, matching the renderer's pre-Material
            // behaviour. `Material::pbr` (and `Material::custom`) flip
            // it to `false`, which engages standard back-face culling —
            // the glTF 2.0 default for single-sided materials.
            cull_mode: if double_sided {
                None
            } else {
                Some(wgpu::Face::Back)
            },
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            // Blend disables depth-write so translucent fragments don't
            // occlude later geometry in the same pass. Opaque / Mask keep
            // depth-write on.
            depth_write_enabled: Some(matches!(
                alpha_mode,
                crate::material::AlphaMode::Opaque | crate::material::AlphaMode::Mask
            )),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    RenderPipeline {
        pipeline,
        bind_group_layouts: layouts.clone(),
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
    use crate::TextureOptions;

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
                .create_texture((&bytes[..], crate::Size::from((2u32, 2u32))))
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
                .create_texture((
                    &bad[..],
                    TextureOptions::from(crate::Size::from((2u32, 2u32))),
                ))
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
                .create_texture((&p, TextureOptions::from(crate::Size::from((1u32, 1u32)))))
                .await;
            assert!(res.is_err());
        });
    }

    // Story: validate() surfaces wgpu validation errors as
    // RendererError::ValidationError instead of leaking through the device's
    // uncaptured-error handler. Repro: a layout that requires one uniform
    // buffer entry, paired with a descriptor that supplies zero entries.
    #[test]
    fn validate_surfaces_wgpu_errors() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let context = renderer.context(None).await.expect("render context");
            let layout =
                context
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("test layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    });

            let result = context.validate("intentionally invalid bind group", || {
                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &layout,
                        entries: &[],
                        label: Some("intentionally invalid bind group"),
                    })
            });

            match result {
                Err(RendererError::ValidationError { label, message }) => {
                    assert_eq!(label, "intentionally invalid bind group");
                    assert!(
                        !message.is_empty(),
                        "expected non-empty validation message, got empty"
                    );
                }
                Err(other) => panic!(
                    "expected ValidationError, got different RendererError: {:?}",
                    other
                ),
                Ok(_) => panic!(
                    "expected ValidationError for layout/entries mismatch, got Ok bind group"
                ),
            }
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

    // E2E: stress set() under contention while rendering to a texture target.
    //
    // Previously flaky (~1-in-3 locally; same race in CI) — two
    // independent failure modes shared distinct root causes:
    //
    //   * `unexpected last time: 0.25` — `set`'s dual write path
    //     (try_write fast path + queue fallback) wasn't ordered against
    //     `flush_pending`. A queued value from an earlier failed
    //     `try_write` could survive in `pending` while later `set` calls
    //     direct-wrote `storage`; the next flush then re-applied the
    //     stale queued value, clobbering the newer direct write.
    //     Fixed by clearing the pending entry for the same key when the
    //     direct-write path succeeds (see `ShaderObject::set`).
    //
    //   * `render: ShaderError(Busy("storage read"))` — the renderer's
    //     bind path used `try_read` to acquire the storage lock for
    //     uniform/storage-buffer reads, surfacing as `Busy` whenever a
    //     concurrent `set` happened to be mid-`try_write`. Reads don't
    //     conflict with each other, and `set` releases the write lock
    //     in microseconds, so the bounded wait of a blocking `read()`
    //     was always the right call. The renderer (and
    //     `Shader::get_uniform[_data]`) now use `read()`.
    //
    // Verified: 50/50 consecutive `cargo test` runs locally. The
    // contention pattern is identical to a real client driving
    // animation uniforms while the renderer renders.
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
                .create_texture((&pixels[..], [2u32, 2u32]))
                .await
                .expect("texture");
            shader.set("tex", &tex).expect("set tex");

            // Render should succeed without panicking
            renderer.render(&shader, &target).expect("render ok");

            let image: Vec<u8> = target.get_image().await;

            assert_eq!(image.len(), 8 * 8 * 4);
        });
    }

    // Story: Bind an R16Unorm texture (prepared mip chain) into a shader and
    // render. This is the consumer's failing path (RemixBrush, FC-BUG report
    // 2026-05-04): without TEXTURE_FORMAT_16BIT_NORM in the device candidates,
    // the texture is silently invalid and `create_view` cascades into an
    // InvalidResource validation error at every frame. With the fix, this
    // round-trips cleanly. textureLoad (no filtering) keeps the layout valid
    // even on devices that lack 16-bit-norm filtering.
    #[test]
    fn render_with_r16unorm_texture_smoke() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8u32, 8u32])
                .await
                .expect("texture target");

            let wgsl = r#"
@group(0) @binding(0) var height: texture_2d<f32>;

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
    let h = textureLoad(height, vec2<i32>(0, 0), 0).r;
    return vec4<f32>(h, h, h, 1.0);
}
            "#;
            let shader = crate::Shader::new(wgsl).expect("shader");

            // 4×4 R16Unorm raw data — same shape as RemixBrush's height tile.
            let words: Vec<u16> = (0..16).map(|i| i * 4096).collect();
            let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
            for w in &words {
                bytes.extend_from_slice(&w.to_le_bytes());
            }
            let chain = crate::texture::Mipmap::build((
                bytes.as_slice(),
                crate::TextureFormat::R16Unorm,
                [4u32, 4u32],
            ))
            .expect("build R16Unorm chain");
            let tex = renderer
                .create_texture(chain)
                .await
                .expect("create R16Unorm texture from prepared chain");
            shader.set("height", &tex).expect("bind R16Unorm");

            renderer
                .render(&shader, &target)
                .expect("render with R16Unorm texture");
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

            let img = target.get_image().await;
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
                let depth_obj = crate::TextureObject::create_depth_texture(&ctx, size, sc);
                let depth_obj = std::sync::Arc::new(depth_obj);
                let depth_id = ctx.register_texture(depth_obj.clone());

                let shader = Shader::default();
                let pass = Pass::from_shader("msaa-depth", &shader);
                // Hold the passes slice to extend its lifetime, then clone the first Arc<PassObject>
                let first_pass = {
                    let passes = pass.passes();
                    passes.first().cloned().expect("pass")
                };
                first_pass.set_depth_target(depth_id);

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

    // Story: acquire_frame retries once on Lost/Outdated and returns other errors as-is.
    #[test]
    fn acquire_frame_exercises_paths() {
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
                VecDeque<Result<Box<dyn crate::TargetFrame>, crate::SurfaceError>>,
            >,
        }
        impl DummyTarget {
            fn new(seq: Vec<Result<Box<dyn crate::TargetFrame>, crate::SurfaceError>>) -> Self {
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
            fn get_current_frame(
                &self,
            ) -> Result<Box<dyn crate::TargetFrame>, crate::SurfaceError> {
                self.seq
                    .write()
                    .pop_front()
                    .unwrap_or_else(|| Ok(Box::new(DummyFrame)))
            }
            async fn get_image(&self) -> Vec<u8> {
                Vec::new()
            }
        }

        pollster::block_on(async move {
            let (_adapter, device, queue) = create_device_and_queue().await;
            let ctx = RenderContext::new(device, queue);

            // Case 1: Lost then Ok -> success
            let t1 = DummyTarget::new(vec![
                Err(crate::SurfaceError::Lost),
                Ok(Box::new(DummyFrame)),
            ]);
            let f1 = ctx.acquire_frame(&t1);
            assert!(f1.is_ok());

            // Case 2: OutOfMemory -> error returned
            let t2 = DummyTarget::new(vec![Err(crate::SurfaceError::OutOfMemory)]);
            let f2 = ctx.acquire_frame(&t2);
            assert!(matches!(f2, Err(crate::SurfaceError::OutOfMemory)));

            // Case 3: Outdated then Timeout -> returns second error
            let t3 = DummyTarget::new(vec![
                Err(crate::SurfaceError::Outdated),
                Err(crate::SurfaceError::Timeout),
            ]);
            let f3 = ctx.acquire_frame(&t3);
            assert!(matches!(f3, Err(crate::SurfaceError::Timeout)));
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
            let depth_obj = crate::TextureObject::create_depth_texture(&ctx, size, 1);
            let depth_obj = std::sync::Arc::new(depth_obj);
            let depth_id = ctx.register_texture(depth_obj.clone());

            let shader = Shader::default();
            let pass = Pass::from_shader("msaa-depth-mismatch", &shader);
            let first_pass = {
                let passes = pass.passes();
                passes.first().cloned().expect("pass")
            };
            first_pass.set_depth_target(depth_id);

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
