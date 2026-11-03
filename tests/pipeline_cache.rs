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

/// The two triangles of a rectangle from `left` to `right` across the whole height.
fn column(left: f32, right: f32) -> [[f32; 2]; 6] {
    [
        [left, -1.0],
        [right, -1.0],
        [right, 1.0],
        [left, -1.0],
        [right, 1.0],
        [left, 1.0],
    ]
}

/// The RGBA bytes at column `x` of the first row of `image`.
fn pixel_at(image: &[u8], x: usize) -> Vec<u8> {
    image[x * 4..x * 4 + 4].to_vec()
}

// Story: One shader draws meshes that lay out their vertices differently, in one
// pass, each with a pipeline of its own layout, where all of them read the first one's.
#[test]
fn one_shader_draws_meshes_with_different_layouts() {
    pollster::block_on(async move {
        // Arrange: three columns feed `color` with different layouts.
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([12, 2])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(VERTEX_COLOR).expect("shader failed");

        // position + color, stride 24
        let red = Mesh::new();
        red.add_vertices(
            column(-1.0, -1.0 / 3.0).map(|p| Vertex::new(p).set("color", [1.0, 0.0, 0.0, 1.0])),
        );

        // position + aux + color, stride 32, color at offset 16
        let green = Mesh::new();
        green.add_vertices(column(-1.0 / 3.0, 1.0 / 3.0).map(|p| {
            Vertex::new(p)
                .set("aux", [0.0, 0.0])
                .set("color", [0.0, 1.0, 0.0, 1.0])
        }));

        // position per vertex, color per instance
        let blue = Mesh::new();
        blue.add_vertices(column(1.0 / 3.0, 1.0).map(Vertex::new));
        blue.add_instance(Instance::new().set("color", [0.0, 0.0, 1.0, 1.0]));
        for mesh in [&red, &green, &blue] {
            shader.add_mesh(mesh).expect("mesh is compatible");
        }

        // Act
        renderer
            .render(&Pass::from_shader("columns", &shader), &target)
            .expect("render failed");
        let image = target.get_image().await.expect("read back");

        // Assert
        assert_eq!(pixel_at(&image, 2), [255, 0, 0, 255], "stride 24");
        assert_eq!(pixel_at(&image, 6), [0, 255, 0, 255], "stride 32");
        assert_eq!(pixel_at(&image, 10), [0, 0, 255, 255], "instance color");
    });
}

// Story: A mesh whose instances are narrower than the first mesh's draws every
// one of them, where the first mesh's stride left room for only two.
#[test]
fn meshes_with_other_instance_strides_draw_every_instance() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([12, 2])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(VERTEX_COLOR).expect("shader failed");

        // One instance of color + tint, stride 32
        let wide = Mesh::new();
        wide.add_vertices(column(-1.0, 0.0).map(Vertex::new));
        wide.add_instance(
            Instance::new()
                .set("color", [1.0, 0.0, 0.0, 1.0])
                .set("tint", [1.0, 1.0, 1.0, 1.0]),
        );

        // Four instances of color, stride 16, the last one blue
        let narrow = Mesh::new();
        narrow.add_vertices(column(0.0, 1.0).map(Vertex::new));
        let colors = [[0.0, 1.0, 0.0, 1.0]; 3]
            .into_iter()
            .chain([[0.0, 0.0, 1.0, 1.0]])
            .map(|color| Instance::new().set("color", color));
        narrow.add_instances(colors);
        shader.add_mesh(&wide).expect("mesh is compatible");
        shader.add_mesh(&narrow).expect("mesh is compatible");

        // Act
        let rendered = renderer.render(&Pass::from_shader("instances", &shader), &target);
        let image = target.get_image().await.expect("read back");

        // Assert
        assert!(rendered.is_ok(), "got {rendered:?}");
        assert_eq!(pixel_at(&image, 3), [255, 0, 0, 255], "the wide mesh");
        assert_eq!(
            pixel_at(&image, 9),
            [0, 0, 255, 255],
            "the fourth narrow instance"
        );
    });
}
