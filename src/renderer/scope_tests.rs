// Tests that render returns the GPU errors its error scopes catch.
use super::*;
use crate::{Instance, Mesh, Shader, Vertex};
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;

/// The fragment stage reads `@location(0)`, which the vertex stage never writes.
/// Naga accepts each entry point alone, and only the pipeline build fails.
const UNFED_FRAGMENT: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}
@fragment fn fs_main(@location(0) tint: vec4<f32>) -> @location(0) vec4<f32> { return tint; }
"#;

/// A workgroup far past every device's invocation limit.
const OVERSIZED_WORKGROUP: &str = r#"
@compute @workgroup_size(4096, 4096, 64)
fn cs_main() {}
"#;

/// Reads a per-vertex position and a per-instance offset.
const OFFSET_INSTANCES: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>) -> VOut {
  return VOut(vec4<f32>(pos + offset, 0.0, 1.0));
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

const TRIANGLE: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];

/// Collects every GPU error the device reports outside an error scope.
fn record_uncaptured(context: &RenderContext) -> Uncaptured {
    let uncaptured = Arc::new(Mutex::new(Vec::new()));
    let sink = uncaptured.clone();
    context
        .device
        .on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            sink.lock().push(error.to_string())
        }));
    uncaptured
}

/// Whether `result` is a validation error filed under `expected`.
fn is_validation(result: &Result<(), RendererError>, expected: &str) -> bool {
    matches!(result, Err(RendererError::DeviceError { kind: ErrorKind::Validation, label, .. }) if label == expected)
}

/// Raises one validation error outside any renderer scope.
fn raise_uncaptured(context: &RenderContext) {
    raise_named(context, "missing its uniform");
}

/// Raises one validation error whose message names the bind group `name`.
fn raise_named(context: &RenderContext, name: &str) {
    let layout = context
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("one uniform"),
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
    let _ = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &layout,
            entries: &[],
        });
}

/// How often a `SpyTarget` handed out a frame, and how often one was presented.
#[derive(Default)]
struct Counts {
    acquired: AtomicUsize,
    presented: AtomicUsize,
}

impl Counts {
    fn acquired(&self) -> usize {
        self.acquired.load(Ordering::SeqCst)
    }

    fn presented(&self) -> usize {
        self.presented.load(Ordering::SeqCst)
    }
}

/// A frame that presents like a window's and counts each present.
struct SpyFrame {
    view: wgpu::TextureView,
    counts: Arc<Counts>,
}

impl TargetFrame for SpyFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }

    fn size(&self) -> wgpu::Extent3d {
        SPY_SIZE
    }

    fn present(self: Box<Self>, _queue: &wgpu::Queue) {
        self.counts.presented.fetch_add(1, Ordering::SeqCst);
    }
}

/// A target whose frames count how often they are acquired and presented.
struct SpyTarget {
    view: wgpu::TextureView,
    counts: Arc<Counts>,
}

const SPY_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 8,
    height: 8,
    depth_or_array_layers: 1,
};

impl SpyTarget {
    /// A target that draws into an `Rgba8Unorm` texture with `usage`.
    fn new(context: &RenderContext, usage: wgpu::TextureUsages) -> Self {
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spy frame"),
            size: SPY_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            counts: Arc::default(),
        }
    }
}

impl crate::target::FrameSource for SpyTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        self.counts.acquired.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(SpyFrame {
            view: self.view.clone(),
            counts: self.counts.clone(),
        }))
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        None
    }
}

impl Target for SpyTarget {
    fn size(&self) -> Size {
        Size::from((SPY_SIZE.width, SPY_SIZE.height))
    }

    fn resize(&mut self, _size: impl Into<Size>) -> Result<(), crate::texture::TextureError> {
        Ok(())
    }

    async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
        Ok(Vec::new())
    }
}

/// The GPU errors the device reported outside an error scope.
type Uncaptured = Arc<Mutex<Vec<String>>>;

/// A renderer with a context and a sink for the GPU errors no scope catches.
async fn setup() -> Result<(Renderer, Arc<RenderContext>, Uncaptured), InitializationError> {
    let renderer = Renderer::new();
    let context = renderer.context(None).await?;
    let uncaptured = record_uncaptured(&context);
    Ok((renderer, context, uncaptured))
}

// Story: a render pipeline that fails validation returns the error on every frame.
// The failed build stays out of the cache, so the next frame builds it again,
// and the scope it ran in is popped, so later errors reach the handler again.
#[test]
fn failed_render_pipeline_returns_an_error_each_frame() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let context = renderer.context(None).await.expect("render context");
        let uncaptured = record_uncaptured(&context);
        let shader = Shader::new(UNFED_FRAGMENT).expect("naga accepts each stage alone");

        // Act
        let first = renderer.render(&shader, &target);
        let second = renderer.render(&shader, &target);

        // Assert
        assert!(is_validation(&first, "render pipeline"), "got {first:?}");
        assert!(is_validation(&second, "render pipeline"), "got {second:?}");
        assert!(context.render_pipelines.is_empty(), "nothing was cached");
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());

        raise_uncaptured(&context);
        assert_eq!(uncaptured.lock().len(), 1, "no scope was left pushed");
    });
}

// Story: a compute pipeline that fails validation returns the error on every
// dispatch, and the failed build stays out of the cache.
#[test]
fn failed_compute_pipeline_returns_an_error_each_frame() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let context = renderer.context(None).await.expect("render context");
        let uncaptured = record_uncaptured(&context);
        let shader = Shader::new(OVERSIZED_WORKGROUP).expect("naga checks no device limit");

        // Act
        let first = renderer.render(&shader, &target);
        let second = renderer.render(&shader, &target);

        // Assert
        assert!(is_validation(&first, "compute pipeline"), "got {first:?}");
        assert!(is_validation(&second, "compute pipeline"), "got {second:?}");
        assert!(context.compute_pipelines.is_empty(), "nothing was cached");
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());

        raise_uncaptured(&context);
        assert_eq!(uncaptured.lock().len(), 1, "no scope was left pushed");
    });
}

// Story: an error that only the recorded commands reveal returns from render, labeled
// as the frame, and the frame is still presented, so a window gets its image back.
// The frame's texture can't be a render attachment, which no library target allows.
#[test]
fn failed_frame_returns_an_error_and_presents() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let target = SpyTarget::new(&context, usage);
        let shader = Shader::default();

        // Act
        let first = renderer.render(&shader, &target);
        let second = renderer.render(&shader, &target);

        // Assert
        assert!(is_validation(&first, "frame"), "got {first:?}");
        assert!(is_validation(&second, "frame"), "got {second:?}");
        assert_eq!(target.counts.presented(), 2, "each failed frame presents");
        assert_eq!(context.render_pipelines.len(), 1, "the pipeline is valid");
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());

        raise_uncaptured(&context);
        assert_eq!(uncaptured.lock().len(), 1, "no scope was left pushed");
    });
}

// Story: a valid frame presents once, which shows the spy target counts presents.
#[test]
fn valid_frame_presents_once() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let target = SpyTarget::new(&context, wgpu::TextureUsages::RENDER_ATTACHMENT);

        // Act
        let result = renderer.render(&Shader::default(), &target);

        // Assert
        assert!(result.is_ok(), "got {result:?}");
        assert_eq!(target.counts.acquired(), 1);
        assert_eq!(target.counts.presented(), 1);
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a CPU error found after the frame was acquired still presents it.
// The pass draws into a texture the renderer no longer holds.
#[test]
fn frame_that_fails_after_acquire_presents() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, _uncaptured) = setup().await.expect("render context");
        let target = SpyTarget::new(&context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let offscreen = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let shader = Shader::default();
        let pass = crate::Pass::from_shader("offscreen", &shader);
        pass.set_target(&offscreen)
            .expect("a texture target is a color target");
        renderer
            .unregister_texture(*offscreen.texture().id())
            .expect("the texture is registered");

        // Act
        let result = renderer.render(&pass, &target);

        // Assert
        assert!(
            matches!(result, Err(RendererError::TextureNotFoundError(_))),
            "got {result:?}"
        );
        assert_eq!(target.counts.acquired(), 1);
        assert_eq!(target.counts.presented(), 1, "the failed frame presents");
    });
}

// Story: a shader that reads vertex inputs and has no mesh fails before any frame is
// acquired, so a window keeps its last image.
#[test]
fn shader_without_its_mesh_acquires_no_frame() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, _uncaptured) = setup().await.expect("render context");
        let target = SpyTarget::new(&context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let shader = Shader::new(OFFSET_INSTANCES).expect("shader");

        // Act
        let result = renderer.render(&shader, &target);

        // Assert
        assert!(
            matches!(
                result,
                Err(RendererError::ShaderError(crate::ShaderError::InvalidKey(
                    _
                )))
            ),
            "got {result:?}"
        );
        assert_eq!(target.counts.acquired(), 0, "no frame was acquired");
    });
}

// Story: a mesh asked for more instances than it holds fails before any frame is acquired.
#[test]
fn instance_count_past_the_rows_acquires_no_frame() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, _uncaptured) = setup().await.expect("render context");
        let target = SpyTarget::new(&context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let shader = Shader::new(OFFSET_INSTANCES).expect("shader");
        let mesh = Mesh::new();
        mesh.add_vertices(TRIANGLE.map(Vertex::new));
        mesh.add_instances([0.0, 1.0].map(|shift| Instance::new().set("offset", [shift, 0.0])));
        mesh.set_instance_count(4);
        shader.add_mesh(&mesh).expect("mesh attaches");

        // Act
        let result = renderer.render(&shader, &target);

        // Assert
        assert!(
            matches!(
                result,
                Err(RendererError::MeshError(
                    crate::mesh::MeshError::InvalidInstance(_)
                ))
            ),
            "got {result:?}"
        );
        assert_eq!(target.counts.acquired(), 0, "no frame was acquired");
    });
}

/// Reads the storage `Lights` root, and draws the light count.
const LIT_FULLSCREEN: &str = r#"
struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32, position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }
struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
@group(0) @binding(1) var<storage, read> lights: Lights;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(f32(lights.count) / 255.0); }
"#;

// Story: a lit shader that holds more lights than the device takes fails before any
// frame is acquired. The lights reach it before the device's cap is known.
#[test]
fn lights_past_the_cap_acquire_no_frame() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, _uncaptured) = setup().await.expect("render context");
        let target = SpyTarget::new(&context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let shader = Shader::new(LIT_FULLSCREEN).expect("shader");
        let pass = crate::Pass::from_shader("lit", &shader);
        let lights: Vec<crate::Light> = (0..3)
            .map(|index| crate::Light::point([index as f32, 0.0, 0.0], [1.0; 3]))
            .collect();
        for light in &lights {
            pass.add(light).expect("under the process cap");
        }
        *context.light_path.write() = LightPath::Uniform { count: 2 };

        // Act
        let result = renderer.render(&pass, &target);

        // Assert
        assert!(
            matches!(
                result,
                Err(RendererError::PassError(
                    crate::PassError::LightCapReached { cap: 2 }
                ))
            ),
            "got {result:?}"
        );
        assert_eq!(target.counts.acquired(), 0, "no frame was acquired");
    });
}

// Story: when an inner scope fails because of an error the outer scope caught
// first, validate reports that first error, under the outer label.
#[test]
fn outer_error_reports_the_root_cause() {
    pollster::block_on(async move {
        // Arrange
        let (_renderer, context, uncaptured) = setup().await.expect("render context");

        // Act
        let result = context.validate("outer", || {
            raise_named(&context, "the cause");
            context.validate("inner", || {
                raise_named(&context, "the symptom");
                Ok(())
            })
        });

        // Assert
        assert!(
            matches!(&result, Err(RendererError::DeviceError { label, detail, .. })
                if label == "outer" && detail.contains("the cause")),
            "got {result:?}"
        );
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

/// A window whose surface fails validation: wgpu sends the details
/// to the device, and the frame only learns that it failed.
struct InvalidSurface {
    context: Arc<RenderContext>,
}

impl crate::target::FrameSource for InvalidSurface {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        raise_named(&self.context, "the surface detail");
        Err(crate::SurfaceError::Validation)
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        Some(self.context.clone())
    }
}

impl Target for InvalidSurface {
    fn size(&self) -> Size {
        Size::from((SPY_SIZE.width, SPY_SIZE.height))
    }

    fn resize(&mut self, _size: impl Into<Size>) -> Result<(), crate::texture::TextureError> {
        Ok(())
    }

    async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
        Ok(Vec::new())
    }
}

// Story: a surface that fails validation reports the details the device caught,
// under the frame's label, where render returned a bare surface error before.
#[test]
fn surface_validation_reports_the_device_error() {
    pollster::block_on(async move {
        // Arrange
        let (renderer, context, uncaptured) = setup().await.expect("render context");
        let target = InvalidSurface {
            context: context.clone(),
        };

        // Act
        let result = renderer.render(&Shader::default(), &target);

        // Assert
        assert!(
            matches!(&result, Err(RendererError::DeviceError { label, detail, .. })
                if label == "frame" && detail.contains("the surface detail")),
            "got {result:?}"
        );
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: a CPU error from the operation still wins over a GPU error the scope caught,
// since it names the input to fix.
#[test]
fn cpu_error_wins_over_the_scope() {
    pollster::block_on(async move {
        // Arrange
        let (_renderer, context, uncaptured) = setup().await.expect("render context");

        // Act
        let result: Result<(), RendererError> = context.validate("outer", || {
            raise_named(&context, "the gpu error");
            Err(crate::ShaderError::UniformNotFound("tint".into()).into())
        });

        // Assert
        assert!(
            matches!(result, Err(RendererError::ShaderError(_))),
            "got {result:?}"
        );
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());
    });
}

// Story: every scope catches internal and out of memory errors along with validation
// errors, so a backend compiler that rejects a pipeline keeps it out of the cache too.
// Each filter reports its own kind.
#[test]
fn scopes_catch_every_error_kind() {
    // Arrange
    let expected = [
        (wgpu::ErrorFilter::OutOfMemory, ErrorKind::Memory),
        (wgpu::ErrorFilter::Internal, ErrorKind::Internal),
        (wgpu::ErrorFilter::Validation, ErrorKind::Validation),
    ];

    // Act
    let missing: Vec<_> = expected
        .iter()
        .filter(|scope| !SCOPES.contains(scope))
        .collect();

    // Assert
    assert!(missing.is_empty(), "no scope catches {missing:?}");
}
