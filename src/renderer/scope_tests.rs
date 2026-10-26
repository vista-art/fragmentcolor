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
    matches!(result, Err(RendererError::ValidationError { label, .. }) if label == expected)
}

/// Raises one validation error outside any renderer scope.
fn raise_uncaptured(context: &RenderContext) {
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
            label: Some("missing its uniform"),
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

// Story: every scope catches internal and out of memory errors along with validation
// errors, so a backend compiler that rejects a pipeline keeps it out of the cache too.
#[test]
fn scopes_catch_every_error_kind() {
    // Arrange
    let expected = [
        wgpu::ErrorFilter::OutOfMemory,
        wgpu::ErrorFilter::Internal,
        wgpu::ErrorFilter::Validation,
    ];

    // Act
    let missing: Vec<_> = expected
        .iter()
        .filter(|filter| !SCOPES.contains(filter))
        .collect();

    // Assert
    assert!(missing.is_empty(), "no scope catches {missing:?}");
}
