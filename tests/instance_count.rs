#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Instance, Mesh, Pass, Renderer, Shader, Target, Vertex};

// Every instance covers the screen, so the last instance drawn sets the pixel.
const INSTANCE_COLOR: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(position, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
"#;

const FULLSCREEN: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

fn colored(colors: &[[f32; 4]]) -> Vec<Instance> {
    colors
        .iter()
        .map(|color| Instance::new().set("color", *color))
        .collect()
}

async fn first_pixel(renderer: &Renderer, shader: &Shader) -> Vec<u8> {
    let target = renderer
        .create_texture_target([8, 8])
        .await
        .expect("create_texture_target failed");
    renderer
        .render(&Pass::from_shader("instances", shader), &target)
        .expect("render failed");
    target.get_image().await[..4].to_vec()
}

// Story: A mesh draws as many instances as it holds, after the list grows and after it shrinks.
#[test]
fn instance_count_follows_the_rows() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let shader = Shader::new(INSTANCE_COLOR).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(Vertex::new));
        mesh.add_instances(colored(&[RED]));
        shader.add_mesh(&mesh).expect("mesh is compatible");

        // Act
        let one = first_pixel(&renderer, &shader).await;

        mesh.add_instances(colored(&[GREEN, BLUE]));
        let grown = first_pixel(&renderer, &shader).await;

        // Two rows fit in the buffer that held three, so it is reused.
        mesh.clear_instances();
        mesh.add_instances(colored(&[RED, GREEN]));
        let shrunk = first_pixel(&renderer, &shader).await;

        // Assert
        assert_eq!(one, [255, 0, 0, 255], "one instance draws red");
        assert_eq!(
            grown,
            [0, 0, 255, 255],
            "the third instance draws blue last"
        );
        assert_eq!(
            shrunk,
            [0, 255, 0, 255],
            "the second instance draws green last"
        );
    });
}
