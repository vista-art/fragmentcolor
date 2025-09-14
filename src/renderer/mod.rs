use crate::{PassObject, ShaderHash, Target, TargetFrame, TextureTarget, shader::Uniform};
use crate::{Size, WindowTarget};
use dashmap::DashMap;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
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

pub mod texture;
pub use texture::*;

mod buffer_pool;
use buffer_pool::BufferPool;

mod readback_pool;
use readback_pool::ReadbackBufferPool;

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

    // Effective MSAA sample count negotiated for current target/format
    sample_count: AtomicU32,

    // Cache RenderPipelines by (shader hash, target format, sample count)
    render_pipelines: DashMap<(ShaderHash, wgpu::TextureFormat, u32), RenderPipeline>,
    buffer_pool: RwLock<BufferPool>,
    pub(crate) readback_pool: RwLock<ReadbackBufferPool>,
    //
    // @TODO
    // _compute_pipelines: DashMap<String, wgpu::ComputePipeline>,
    // _textures: DashMap<String, wgpu::Texture>,
    // _samplers: DashMap<String, wgpu::Sampler>,
}

impl RenderContext {
    /// Creates a new Context with the given device and queue.
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let buffer_pool = BufferPool::new_uniform_pool("Uniform Buffer Pool", &device);

        RenderContext {
            device,
            queue,

            sample_count: AtomicU32::new(1),
            render_pipelines: DashMap::new(),
            buffer_pool: RwLock::new(buffer_pool),
            readback_pool: RwLock::new(ReadbackBufferPool::new("Readback Buffer Pool", 8)),
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
    pub fn set_sample_count(&self, n: u32) {
        self.sample_count.store(n.max(1), Ordering::Relaxed);
    }

    pub fn sample_count(&self) -> u32 {
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
        let mut _msaa_tex: Option<wgpu::Texture> = None;
        let mut _msaa_view: Option<wgpu::TextureView> = None;

        if sc > 1 {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Color"),
                size,
                mip_level_count: 1,
                sample_count: sc,
                dimension: wgpu::TextureDimension::D2,
                format: fmt,
                view_formats: &[],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
            let v = tex.create_view(&wgpu::TextureViewDescriptor::default());
            _msaa_view = Some(v);
            _msaa_tex = Some(tex);
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
            let cached = self
                .render_pipelines
                .entry((shader.hash, format, sc))
                .or_insert_with(|| {
                    let layouts =
                        create_bind_group_layouts(&self.device, &shader.storage.read().uniforms);
                    let pipeline =
                        create_render_pipeline(&self.device, &layouts, shader, format, sc);

                    RenderPipeline {
                        pipeline,
                        bind_group_layouts: layouts,
                    }
                });

            let mut bind_group_entries: HashMap<u32, Vec<wgpu::BindGroupEntry>> = HashMap::new();
            let mut buffer_locations = Vec::new();

            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                let storage = shader.storage.read();
                let bytes = storage
                    .get_bytes(name)
                    .ok_or(crate::ShaderError::UniformNotFound(name.clone()))?;

                let buffer_location = {
                    let mut buffer_pool = self.buffer_pool.write();
                    buffer_pool.upload(bytes, &self.queue, &self.device)
                };

                buffer_locations.push((uniform, buffer_location));
            }

            let buffer_pool = self.buffer_pool.read();
            for (uniform, location) in buffer_locations {
                let binding = buffer_pool.get_binding(location);

                bind_group_entries
                    .entry(uniform.group)
                    .or_default()
                    .push(wgpu::BindGroupEntry {
                        binding: uniform.binding,
                        resource: wgpu::BindingResource::Buffer(binding),
                    });
            }

            // Build bind groups per layout (by group index)
            let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
            for (group, entries) in bind_group_entries {
                let Some(layout) = cached.bind_group_layouts.get(&group) else {
                    return Err(RendererError::BindGroupLayoutError(format!(
                        "Missing bind group layout for group {}",
                        group
                    )));
                };
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
            // render_pass.set_blend_constant(color);
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
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

        let entry = wgpu::BindGroupLayoutEntry {
            binding: uniform.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(min_size),
            },
            count: None,
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
            buffers: &[],
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
mod readback_pool_tests;

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
            assert!(ctx.render_pipelines.contains_key(&(shader_hash, fmt, 1)));

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
                assert!(ctx.render_pipelines.contains_key(&(shader_hash, fmt, sc2)));
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
}
