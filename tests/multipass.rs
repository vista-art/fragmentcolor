#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Scene, Shader, Target};

const FILL: &str = r#"
struct VertexOutput { @builtin(position) position: vec4<f32> }

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    return VertexOutput(vec4<f32>(p[i], 0.0, 1.0));
}

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> { return color; }
"#;

const COPY_UNIFORM: &str = r#"
@group(0) @binding(0) var<uniform> value: vec4<f32>;
@group(0) @binding(1) var<storage, read_write> multipass_copy: vec4<f32>;

@compute @workgroup_size(1)
fn cs_main() { multipass_copy = value; }
"#;

/// A compute shader that adds 1 to the storage root `name` each time it runs.
fn counter(name: &str) -> Shader {
    let source = format!(
        "@group(0) @binding(0) var<storage, read_write> {name}: array<u32, 1>;
@compute @workgroup_size(1)
fn cs_main() {{ {name}[0] += 1u; }}
"
    );
    Shader::new(&source).expect("counter shader failed")
}

fn fill(color: [f32; 4]) -> Shader {
    let shader = Shader::new(FILL).expect("fill shader failed");
    shader.set("color", color).expect("set failed");
    shader
}

fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

// Story: Two render passes in one render call each draw with their own uniform values.
#[test]
fn render_passes_keep_their_own_uniforms() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let side = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");

        let red = fill([1.0, 0.0, 0.0, 1.0]);
        let first = Pass::from_shader("first", &red);
        first.set_target(&side).expect("set_target failed");

        let green = fill([0.0, 1.0, 0.0, 1.0]);
        let second = Pass::from_shader("second", &green);

        // Act
        renderer
            .render(&vec![&first, &second], &probe)
            .expect("render failed");

        // Assert
        let first_pixels = side.get_image().await.expect("read back");
        let second_pixels = probe.get_image().await.expect("read back");
        assert_eq!(&first_pixels[..4], &[255, 0, 0, 255], "first pass color");
        assert_eq!(&second_pixels[..4], &[0, 255, 0, 255], "second pass color");
    });
}

// Story: A compute pass reads its own uniform when a render pass follows it in one render call.
#[test]
fn compute_pass_keeps_its_uniform_before_a_render_pass() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");

        let copy = Shader::new(COPY_UNIFORM).expect("compute shader failed");
        copy.set("value", [0.25, 0.5, 0.75, 1.0])
            .expect("set failed");
        let compute = Pass::from_shader("copy", &copy);

        let green = fill([0.0, 1.0, 0.0, 1.0]);
        let draw = Pass::from_shader("draw", &green);

        // Act
        renderer
            .render(&vec![&compute, &draw], &probe)
            .expect("render failed");

        // Assert
        let bytes = renderer
            .read_storage(&copy, "multipass_copy")
            .await
            .expect("read_storage failed");
        assert_eq!(floats(&bytes), vec![0.25, 0.5, 0.75, 1.0]);
        let pixels = probe.get_image().await.expect("read back");
        assert_eq!(&pixels[..4], &[0, 255, 0, 255], "render pass color");
    });
}

// Story: a pass runs once in each render, after the passes it requires,
// whether the render takes a list, a Scene or the pass itself.
#[test]
fn required_passes_run_once() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([1, 1])
            .await
            .expect("create_texture_target failed");
        let first_count = counter("first_runs");
        let second_count = counter("second_runs");
        let first = Pass::from_shader("first", &first_count);
        let second = Pass::from_shader("second", &second_count);
        second.require(&first).expect("second requires first");
        let scene = Scene::new();
        scene.add_pass(&first);
        scene.add_pass(&second);
        let read = async |shader: &Shader, name: &str| {
            let bytes = renderer.read_storage(shader, name).await.expect("read");
            u32::from_le_bytes(bytes[..4].try_into().expect("four bytes"))
        };

        // Act / Assert
        renderer
            .render(&vec![&first, &second], &target)
            .expect("render the list");
        assert_eq!(read(&first_count, "first_runs").await, 1, "a listed pass");
        assert_eq!(read(&second_count, "second_runs").await, 1);

        renderer.render(&scene, &target).expect("render the scene");
        assert_eq!(read(&first_count, "first_runs").await, 2, "a scene member");
        assert_eq!(read(&second_count, "second_runs").await, 2);

        renderer
            .render(&vec![&second], &target)
            .expect("render a list without first");
        assert_eq!(read(&first_count, "first_runs").await, 3, "a required pass");
        assert_eq!(read(&second_count, "second_runs").await, 3);

        renderer
            .render(&vec![&second, &first], &target)
            .expect("render a list out of order");
        assert_eq!(read(&first_count, "first_runs").await, 4, "a listed pass");
        assert_eq!(read(&second_count, "second_runs").await, 4);
    });
}
