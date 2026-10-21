#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Mesh, Pass, Renderer, Shader, Target, Vertex};

// The first pair draws red. The second pair reads a `shift` input the mesh does not have.
const TWO_ENTRY_POINTS: &str = r#"
@vertex
fn vs_first(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@vertex
fn vs_second(@location(0) position: vec2<f32>, @location(1) shift: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position + shift, 0.0, 1.0);
}

@fragment
fn fs_first() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }

@fragment
fn fs_second() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
"#;

const FULLSCREEN: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];

// Story: A module with two vertex and two fragment entry points draws with the first of each.
#[test]
fn render_pipeline_runs_the_first_entry_points() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(TWO_ENTRY_POINTS).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(Vertex::new));
        shader
            .add_mesh(&mesh)
            .expect("mesh matches the first vertex entry point");

        // Act
        renderer
            .render(&Pass::from_shader("first", &shader), &target)
            .expect("render failed");
        let pixel = target.get_image().await[..4].to_vec();

        // Assert
        assert_eq!(pixel, [255, 0, 0, 255], "vs_first and fs_first draw red");
    });
}
