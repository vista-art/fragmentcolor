#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Instance, Mesh, Pass, Renderer, Shader, Target, Vertex};

const VERTEX_COLOR: &str = r#"
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

async fn first_pixel(renderer: &Renderer, mesh: &Mesh) -> Vec<u8> {
    let target = renderer
        .create_texture_target([8, 8])
        .await
        .expect("create_texture_target failed");
    let shader = Shader::new(VERTEX_COLOR).expect("shader failed");
    shader.add_mesh(mesh).expect("mesh is compatible");
    renderer
        .render(&Pass::from_shader("draw", &shader), &target)
        .expect("render failed");
    target.get_image().await.expect("read back")[..4].to_vec()
}

// Story: Shaders with the same source get their own pipeline when their meshes lay out vertices differently.
#[test]
fn same_source_with_different_vertex_layouts() {
    pollster::block_on(async move {
        // Arrange: three meshes feed `color` with different layouts.
        let renderer = Renderer::new();

        // position + color, stride 24
        let red = Mesh::new();
        red.add_vertices(FULLSCREEN.map(|p| Vertex::new(p).set("color", [1.0, 0.0, 0.0, 1.0])));

        // position + aux + color, stride 32, color at offset 16
        let green = Mesh::new();
        green.add_vertices(FULLSCREEN.map(|p| {
            Vertex::new(p)
                .set("aux", [0.0, 0.0])
                .set("color", [0.0, 1.0, 0.0, 1.0])
        }));

        // position per vertex, color per instance
        let blue = Mesh::new();
        blue.add_vertices(FULLSCREEN.map(Vertex::new));
        blue.add_instance(Instance::new().set("color", [0.0, 0.0, 1.0, 1.0]));

        // Act
        let red_pixel = first_pixel(&renderer, &red).await;
        let green_pixel = first_pixel(&renderer, &green).await;
        let blue_pixel = first_pixel(&renderer, &blue).await;

        // Assert
        assert_eq!(red_pixel, [255, 0, 0, 255], "vertex color, stride 24");
        assert_eq!(green_pixel, [0, 255, 0, 255], "vertex color, stride 32");
        assert_eq!(blue_pixel, [0, 0, 255, 255], "instance color");
    });
}
