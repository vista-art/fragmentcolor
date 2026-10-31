//! `RenderContext`: the device, queue, caches and pools one `Renderer` opens,
//! and the calls that open it once for every target and texture of that renderer.

use crate::core::lower_cap;
#[cfg(wasm)]
use crate::core::renderer::device_errors;
use crate::core::renderer::frame::bind_groups::PbrDefaults;
use crate::core::renderer::lights::{self, LightPath};
use crate::core::renderer::pipelines::{ComputePipeline, ComputePipelineKey};
use crate::core::renderer::pipelines::{RenderPipeline, RenderPipelineKey};
use crate::core::renderer::{InitializationError, Renderer, checks, device};
use crate::core::renderer::{
    ReadbackBufferPool, StorageBufferPool, TexturePool, UniformBufferPool,
};
use crate::texture::{TextureId, TextureObject};
use crate::{ShaderHash, ShaderObject};
use dashmap::DashMap;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

#[derive(Debug)]
/// Draws things on the screen or a texture.
///
/// It owns and manages all GPU resources, serving as the
/// main graphics context provider for the application.
pub(crate) struct RenderContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    /// Tells this context from every other one, so a mesh knows which device its
    /// buffers live on. wgpu compares the devices of two instances by an index both reuse.
    pub(crate) id: u64,
    /// The adapter the device came from. A texture checks its format and copies
    /// against what the adapter reports, since GLES and WebGL2 take less than WebGPU.
    pub(crate) adapter: Option<wgpu::Adapter>,

    // Cache for RenderPipelines and ComputePipelines. Shared, so a wasm
    // error that lands after the insert can evict the pipeline it names.
    pub(super) render_pipelines: Arc<DashMap<RenderPipelineKey, RenderPipeline>>,
    pub(super) compute_pipelines: Arc<DashMap<ComputePipelineKey, ComputePipeline>>,

    pub(super) buffer_pool: RwLock<UniformBufferPool>,
    // Storage buffer pool (STORAGE | COPY_DST), separate from uniform pool
    pub(super) storage_pool: RwLock<StorageBufferPool>,

    pub(crate) readback_pool: RwLock<ReadbackBufferPool>,
    pub(crate) texture_pool: RwLock<TexturePool>,

    // Texture registry: id -> TextureObject. An entry stays while its
    // `Registration` lives, so it goes with the last handle to the texture.
    pub(super) textures: DashMap<TextureId, Arc<TextureObject>>,
    /// True on backends that reject uniform bindings not aligned to 16
    /// bytes (OpenGL, WebGL2); shaders then compile a padded module.
    pub(super) aligned_uniform_bindings: AtomicBool,
    /// True on the browser's WebGPU, where wgpu hands the browser WGSL that naga
    /// writes without `@size` and `@align`. A shader whose buffer structs use them
    /// then goes as WGSL that keeps its layout (`layout::pick_source`).
    pub(super) writes_wgsl: AtomicBool,

    // Persistent storage buffer registry: root name -> (buffer, span)
    pub(super) storage_registry: DashMap<String, (wgpu::Buffer, u64)>,
    /// What `check_shader` decided for each shader source on this device,
    /// so a source is checked once, whether it passed or not.
    pub(super) checked: DashMap<ShaderHash, checks::shader::Verdict>,
    /// The device's features, limits and downlevel flags, read once,
    /// since each read crosses into JavaScript on the browser's WebGPU.
    pub(crate) features: wgpu::Features,
    pub(crate) limits: wgpu::Limits,
    pub(crate) downlevel: wgpu::DownlevelFlags,
    /// The adapter's backend, which decides what the GL path can keep.
    pub(super) backend: wgpu::Backend,

    // MSAA sample count negotiated for current target/format
    sample_count: AtomicU32,

    // Lazy-init 1x1 fallback textures for the default PBR shader's glTF
    // texture-map bindings. Populated synchronously on first use inside
    // `process_render_pass`, so `Material::pbr` doesn't need a Renderer.
    pub(super) pbr_defaults: RwLock<Option<Arc<PbrDefaults>>>,

    // Reusable scratch buffers for `build_pass_draws`. Cleared at the
    // start of each call and reused across frames, avoiding the per-frame
    // `Vec` allocations the previous shape paid.
    pub(super) scratch: RwLock<RenderScratch>,

    /// How shaders take their lights on this device: storage roots binned by
    /// a compute pass, or a uniform block. Picked once, when the context opens.
    pub(super) light_path: RwLock<LightPath>,
    /// Each pass's light sets, by pass id and light list. They stay out of the
    /// storage registry and the per-frame pools.
    pub(super) light_sets: RwLock<HashMap<(u64, u64), lights::LightSet>>,
    /// Set when a frame fails, so the next frame packs and bins every light
    /// set again: the failed frame's binning never reached the GPU.
    pub(super) lights_failed: Arc<AtomicBool>,
    /// The light binning pipeline, built on first use.
    pub(super) binning: Arc<RwLock<Option<Arc<lights::BinningPipeline>>>>,
    #[cfg(test)]
    pub(super) light_probe: lights::tests::LightProbe,

    /// The first GPU error a wasm error scope caught after its call returned.
    /// The next render, load or readback returns it.
    #[cfg(wasm)]
    pub(crate) late: Arc<device_errors::late::ErrorSlot>,
}

/// Frame-local scratch allocations cached on `RenderContext`. Cleared at
/// the start of each `build_pass_draws` invocation and reused across
/// frames so the per-frame draw-queue build doesn't churn allocations.
#[derive(Debug, Default)]
pub(super) struct RenderScratch {
    /// Insertion-ordered opaque groups keyed by `(shader_ptr, mesh_ptr)`.
    /// Each entry carries the byte-packed instance buffer for that group.
    pub(super) opaque_groups: Vec<((usize, usize), Vec<u8>)>,
}

impl RenderContext {
    /// Creates a new Context with the given device and queue.
    pub(crate) fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let buffer_pool = UniformBufferPool::new("Uniform Buffer Pool", &device);
        let storage_pool = StorageBufferPool::new("Storage Buffer Pool", &device);
        let readback_pool = ReadbackBufferPool::new("Readback Buffer Pool", 8);
        let limits = device.limits();
        let features = device.features();
        let storage_limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);

        RenderContext {
            device,
            queue,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            adapter: None,

            render_pipelines: Arc::default(),
            compute_pipelines: Arc::default(),

            buffer_pool: RwLock::new(buffer_pool),
            storage_pool: RwLock::new(storage_pool),
            readback_pool: RwLock::new(readback_pool),
            texture_pool: RwLock::new(TexturePool::new(16)),

            textures: DashMap::new(),
            aligned_uniform_bindings: AtomicBool::new(false),
            writes_wgsl: AtomicBool::new(false),
            storage_registry: DashMap::new(),
            checked: DashMap::new(),
            features,
            limits: limits.clone(),
            downlevel: wgpu::DownlevelFlags::all(),
            backend: wgpu::Backend::Noop,
            sample_count: AtomicU32::new(1),
            pbr_defaults: RwLock::new(None),
            scratch: RwLock::new(RenderScratch::default()),
            // `Renderer::context` picks the path from the adapter once it has one.
            light_path: RwLock::new(LightPath::Clustered {
                limit: storage_limit,
            }),
            light_sets: RwLock::new(HashMap::new()),
            lights_failed: Arc::default(),
            binning: Arc::default(),
            #[cfg(test)]
            light_probe: lights::tests::LightProbe::default(),
            #[cfg(wasm)]
            late: Arc::default(),
        }
    }

    /// The persistent buffer behind the storage root `name`.
    /// It is created, or recreated at a new `span`, with the shader's bytes,
    /// and takes a fresh upload only when `set` has changed them since the last frame.
    pub(super) fn storage_buffer(
        &self,
        shader: &ShaderObject,
        name: &str,
        span: u64,
    ) -> wgpu::Buffer {
        let existing = self
            .storage_registry
            .get(name)
            .filter(|entry| entry.value().1 == span)
            .map(|entry| entry.value().0.clone());
        let created = existing.is_none();
        let buffer = existing.unwrap_or_else(|| {
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Persistent Storage Buffer: {name}")),
                size: span,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.storage_registry
                .insert(name.to_string(), (buffer.clone(), span));
            buffer
        });
        // An upgradable read blocks a `set` until the dirty flag is cleared, so no update is lost.
        let storage = shader.storage.upgradable_read();
        let dirty = storage.is_storage_dirty(name);
        if created || dirty {
            match storage.get_bytes(name) {
                Some(bytes) => self.queue.write_buffer(&buffer, 0, bytes),
                None => self
                    .queue
                    .write_buffer(&buffer, 0, &vec![0u8; span as usize]),
            }
        }
        if dirty {
            RwLockUpgradableReadGuard::upgrade(storage).clear_storage_dirty(name);
        }
        buffer
    }

    pub(crate) fn set_sample_count(&self, count: u32) {
        self.sample_count.store(count.max(1), Ordering::Relaxed);
    }

    pub(crate) fn sample_count(&self) -> u32 {
        self.sample_count.load(Ordering::Relaxed).max(1)
    }
}

impl Renderer {
    /// The wgpu instance, created by the first call that needs one.
    pub(super) async fn instance(&self) -> Arc<wgpu::Instance> {
        if let Some(instance) = self.instance.read().as_ref() {
            return instance.clone();
        }
        let _creating = self.instance_lock.lock().await;
        if let Some(instance) = self.instance.read().as_ref() {
            return instance.clone();
        }
        let instance = Arc::new(device::create_instance().await);
        self.instance.write().replace(instance.clone());
        instance
    }

    /// The context every target and texture of this Renderer shares, created once.
    /// A call that races the first one waits for it and takes its context.
    pub(super) async fn context(
        &self,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Arc<RenderContext>, InitializationError> {
        if let Some(context) = self.context.read().as_ref() {
            return Ok(context.clone());
        }
        let _creating = self.context_lock.lock().await;
        let context = if let Some(context) = self.context.read().as_ref() {
            context.clone()
        } else {
            let instance = self.instance().await;
            let adapter = device::request_adapter(instance.as_ref(), surface).await?;
            let (device, queue) = device::request_device(&adapter).await?;
            let mut context = RenderContext::new(device, queue);
            context.adapter = Some(adapter.clone());
            context.downlevel = adapter.get_downlevel_capabilities().flags;
            context.backend = adapter.get_info().backend;
            let context = Arc::new(context);
            let flags = adapter.get_downlevel_capabilities().flags;
            context.aligned_uniform_bindings.store(
                !flags.contains(wgpu::DownlevelFlags::BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED),
                Ordering::Relaxed,
            );
            let browser = adapter.get_info().backend == wgpu::Backend::BrowserWebGpu;
            context.writes_wgsl.store(browser, Ordering::Relaxed);
            let path = LightPath::pick(&adapter, &context.device);
            *context.light_path.write() = path;
            lower_cap(path.cap());

            self.adapter.write().replace(adapter);
            self.context.write().replace(context.clone());

            context
        };

        Ok(context)
    }
}
