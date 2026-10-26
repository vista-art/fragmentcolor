// Tests that render returns the GPU errors its error scopes catch.
use super::*;
use crate::{Instance, Mesh, Pass, Shader, Vertex};
use parking_lot::Mutex;

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
fn record_uncaptured(context: &RenderContext) -> Arc<Mutex<Vec<String>>> {
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

// Story: an error that only the recorded commands reveal returns from render.
// Two meshes share one pipeline built from the first mesh's 24-byte instance
// stride, so the second mesh's four 8-byte rows hold only two instances.
#[test]
fn failed_frame_commands_return_an_error() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8u32, 8u32])
            .await
            .expect("texture target");
        let context = renderer.context(None).await.expect("render context");
        let uncaptured = record_uncaptured(&context);
        let shader = Shader::new(OFFSET_INSTANCES).expect("shader");
        let tinted = Mesh::new();
        tinted.add_vertices(TRIANGLE.map(Vertex::new));
        tinted.add_instance(
            Instance::new()
                .set("offset", [0.0, 0.0])
                .set("tint", [1.0, 1.0, 1.0, 1.0]),
        );
        let plain = Mesh::new();
        plain.add_vertices(TRIANGLE.map(Vertex::new));
        plain.add_instances(
            [0.0, 1.0, 2.0, 3.0].map(|shift| Instance::new().set("offset", [shift, 0.0])),
        );
        shader.add_mesh(&tinted).expect("tinted mesh attaches");
        shader.add_mesh(&plain).expect("plain mesh attaches");
        let pass = Pass::from_shader("instances", &shader);

        // Act
        let first = renderer.render(&pass, &target);
        let second = renderer.render(&pass, &target);

        // Assert
        assert!(is_validation(&first, "frame"), "got {first:?}");
        assert!(is_validation(&second, "frame"), "got {second:?}");
        assert_eq!(
            context.render_pipelines.len(),
            1,
            "the pipeline itself is valid"
        );
        assert!(uncaptured.lock().is_empty(), "got {:?}", uncaptured.lock());

        raise_uncaptured(&context);
        assert_eq!(uncaptured.lock().len(), 1, "no scope was left pushed");
    });
}
