#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{
    AlphaMode, Camera, Material, Mesh, Model, Pass, Renderer, Scene, Shader, Target, TextureTarget,
    Vertex,
};

const SIZE: u32 = 16;

// A fullscreen triangle in plain green, at the near plane.
const BACKDROP: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
"#;

// Colored vertices drawn where they sit in clip space.
const PAINT: &str = r#"
struct VOut { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> };

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> VOut {
    var out: VOut;
    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
"#;

const RED: [f32; 3] = [1.0, 0.0, 0.0];
const BLUE: [f32; 3] = [0.0, 0.0, 1.0];

/// A one-meter square facing the camera at depth `z`, drawn with `material`.
fn square(material: Material, z: f32) -> Model {
    let mesh = Mesh::new();
    let corners = [
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];
    mesh.add_vertices(
        corners.map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    let model = Model::new(mesh, material);
    model.translate([0.0, 0.0, z]);
    model
}

/// A square at depth `z` glowing in `color`.
fn glow(color: [f32; 3], z: f32) -> Model {
    let material = Material::pbr()
        .base_color([0.0, 0.0, 0.0, 1.0])
        .emissive(color);
    square(material, z)
}

/// A square at depth `z` of half-clear glass glowing in `color`.
fn glass(color: [f32; 3], z: f32) -> Model {
    let material = Material::pbr()
        .base_color([0.0, 0.0, 0.0, 0.5])
        .emissive(color)
        .alpha_mode(AlphaMode::Blend);
    square(material, z)
}

fn camera() -> Camera {
    Camera::perspective(1.0, 1.0, 0.1, 100.0)
        .expect("camera")
        .look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .expect("camera")
}

async fn target(renderer: &Renderer, side: u32) -> TextureTarget {
    renderer
        .create_texture_target([side, side])
        .await
        .expect("create_texture_target failed")
}

/// The RGBA bytes at `(x, y)` of an image `side` pixels wide.
fn pixel(image: &[u8], side: u32, x: u32, y: u32) -> [u8; 4] {
    let start = ((y * side + x) * 4) as usize;
    [
        image[start],
        image[start + 1],
        image[start + 2],
        image[start + 3],
    ]
}

// Story: overlapping models draw with depth, so the near one shows even when
// the far one comes later.
#[test]
fn the_nearest_model_shows_whatever_the_order() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let scene = Scene::new();
        scene.add(&glow(RED, 0.0)).expect("add failed");
        scene.add(&glow(BLUE, -2.0)).expect("add failed");
        scene.add(&camera()).expect("add failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&scene, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            red > 200 && blue < 50,
            "the far model drew over the near one: {red}, {blue}"
        );
    });
}

// Story: a fullscreen backdrop in a model pass ignores the depth buffer, so it
// stays behind the models instead of hiding them.
#[test]
fn a_backdrop_stays_behind_the_models() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let backdrop = Shader::new(BACKDROP).expect("Shader::new failed");
        let scene = Scene::new();
        scene.add_pass(&Pass::from_shader("world", &backdrop));
        scene.add_to("world", &glow(RED, 0.0)).expect("add failed");
        scene.add_to("world", &camera()).expect("add failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&scene, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, green, _, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            red > 200 && green < 50,
            "the backdrop hid the model: {red}, {green}"
        );
        let [_, green, _, _] = pixel(&image, SIZE, 0, 0);
        assert!(
            green > 200,
            "the backdrop is missing around the model: {green}"
        );
    });
}

// Story: a pass without models keeps drawing in order, so the later draw wins.
#[test]
fn plain_passes_draw_in_order() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let shader = Shader::new(PAINT).expect("Shader::new failed");
        let mesh = Mesh::new();
        let cover = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        for (z, color) in [(0.2, [1.0, 0.0, 0.0, 1.0]), (0.8, [0.0, 0.0, 1.0, 1.0])] {
            mesh.add_vertices(cover.map(|[x, y]| Vertex::new([x, y, z]).set("color", color)));
        }
        shader.add_mesh(&mesh).expect("mesh is compatible");
        let frame = target(&renderer, SIZE).await;
        renderer
            .render(&Pass::from_shader("paint", &shader), &frame)
            .expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(blue > 200 && red < 50, "the later draw lost: {red}, {blue}");
    });
}

// Story: a model pass that draws into its own target gets a depth texture the
// size of that target, not of the frame passed to render.
#[test]
fn a_pass_target_gets_a_depth_of_its_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let image = target(&renderer, SIZE * 2).await;
        let pass = Pass::new("into the image");
        pass.add(&glow(RED, 0.0)).expect("add failed");
        pass.add(&glow(BLUE, -2.0)).expect("add failed");
        pass.add(&camera()).expect("add failed");
        pass.set_target(&image).expect("set_target failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&pass, &frame).expect("render failed");
        let pixels = image.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&pixels, SIZE * 2, SIZE, SIZE);
        assert!(
            red > 200 && blue < 50,
            "the pass drew nothing or without depth: {red}, {blue}"
        );
    });
}

// Story: a depth target the app sets keeps working as before.
#[test]
fn an_explicit_depth_target_still_works() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let depth = renderer
            .create_depth_texture([SIZE, SIZE])
            .await
            .expect("create_depth_texture failed");
        let pass = Pass::new("with its own depth");
        pass.add(&glow(RED, 0.0)).expect("add failed");
        pass.add(&glow(BLUE, -2.0)).expect("add failed");
        pass.add(&camera()).expect("add failed");
        pass.set_depth_target(&depth)
            .expect("set_depth_target failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&pass, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            red > 200 && blue < 50,
            "the depth target didn't sort: {red}, {blue}"
        );
    });
}

// Story: a shader in a model pass that draws no model, such as an overlay,
// ignores the depth buffer and draws in order, so one added after the models covers them.
#[test]
fn an_overlay_in_a_model_pass_draws_in_order() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let pass = Pass::new("world");
        pass.add(&glow(RED, 0.0)).expect("add failed");
        pass.add(&camera()).expect("add failed");
        let overlay = Shader::new(PAINT).expect("Shader::new failed");
        let mesh = Mesh::new();
        let cover = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(
            cover.map(|[x, y]| Vertex::new([x, y, 0.99]).set("color", [0.0, 0.0, 1.0, 1.0])),
        );
        overlay.add_mesh(&mesh).expect("mesh is compatible");
        pass.add_shader(&overlay).expect("add_shader failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&pass, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            blue > 200 && red < 50,
            "the overlay tested depth: {red}, {blue}"
        );
    });
}

// Story: with a depth target of its own, a model pass still keeps its backdrop
// behind the models.
#[test]
fn a_backdrop_stays_behind_the_models_with_a_depth_target() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let backdrop = Shader::new(BACKDROP).expect("Shader::new failed");
        let pass = Pass::from_shader("world", &backdrop);
        pass.add(&glow(RED, 0.0)).expect("add failed");
        pass.add(&camera()).expect("add failed");
        let depth = renderer
            .create_depth_texture([SIZE, SIZE])
            .await
            .expect("create_depth_texture failed");
        pass.set_depth_target(&depth)
            .expect("set_depth_target failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&pass, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, green, _, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            red > 200 && green < 50,
            "the backdrop hid the model: {red}, {green}"
        );
    });
}

// Story: blended models test depth without writing it, so glass behind an
// opaque model stays hidden.
#[test]
fn glass_behind_an_opaque_model_stays_hidden() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let scene = Scene::new();
        scene.add(&glow(RED, 0.0)).expect("add failed");
        scene.add(&glass(BLUE, -2.0)).expect("add failed");
        scene.add(&camera()).expect("add failed");
        let frame = target(&renderer, SIZE).await;
        renderer.render(&scene, &frame).expect("render failed");
        let image = frame.get_image().await.expect("read back");
        let [red, _, blue, _] = pixel(&image, SIZE, SIZE / 2, SIZE / 2);
        assert!(
            red > 200 && blue < 50,
            "the glass drew over the near model: {red}, {blue}"
        );
    });
}
