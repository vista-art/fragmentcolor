//! Toon shading with ink outlines, drawn by a pass of their own.
//!
//! A `ToonMaterial` shades each shape in flat bands, but no material can draw
//! its outline: whether a pixel sits on a silhouette depends on what lies
//! behind it, which the draw of one surface never sees. So the scene draws in
//! three passes:
//!
//! 1. `normals` draws the world-space normal of every shape into a texture,
//!    and its depth into a depth texture that a later pass can read.
//! 2. `world` draws the same shapes with their toon materials.
//! 3. `outline` draws a fullscreen triangle over the result. It reads both
//!    textures and puts ink where the depth jumps, along a silhouette, and
//!    where the normal turns, along a crease.
//!
//! Each shape is two Models over one Mesh, at the same transform: one with its
//! `ToonMaterial` in `world`, and one with the normals `Shader` in `normals`.
//!
//! Run it in a window, where the camera circles the shapes:
//!   ./fc example toon_outline
//! Or save one frame without a window, into examples/rust/toon_outline.png:
//!   ./fc example toon_outline --png toon_outline.png

// `App::add_state` proves `Viewer: Send + Sync` through the Scene's wgpu
// resources, a chain deeper than the default limit of 128.
#![recursion_limit = "256"]

use fragmentcolor::{
    AlphaMode, App, AppError, Camera, Light, Mesh, Model, Pass, Renderer, Scene, SetupResult,
    Shader, Target, Texture, ToonMaterial, Vertex, call,
};
use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::{Window, WindowAttributes};

const NORMALS: &str = include_str!("shaders/outline_normals.wgsl");
const OUTLINE: &str = include_str!("shaders/outline.wgsl");

/// The camera's near and far planes. The outline reads them to turn a depth
/// back into a distance.
const NEAR: f32 = 0.1;
const FAR: f32 = 50.0;

/// The scene, its camera, and the two textures the outline reads, which
/// follow the size of the window.
struct Stage {
    scene: Scene,
    camera: Camera,
    depth: Texture,
    normals: Texture,
}

/// Builds the three passes and the shapes, for a target of `size` pixels.
/// `scale` is the display's scale factor, which widens the lines to match.
async fn build_stage(renderer: &Renderer, size: [u32; 2], scale: f32) -> Result<Stage, AppError> {
    // The normals pass draws into these, and the outline pass reads them.
    let depth = renderer.create_depth_texture(size).await?;
    let normals = renderer.create_texture_target(size).await?.texture();

    let normals_pass = Pass::new("normals");
    normals_pass.set_target(&normals)?;
    normals_pass.set_depth_target(&depth)?;

    let world = Pass::new("world");
    world.set_clear_color([0.95, 0.92, 0.85, 1.0])?;

    // The outline blends its ink over the colors `world` drew.
    let ink = Shader::new(OUTLINE)?.alpha_mode(AlphaMode::Blend);
    ink.set("depth", &depth)?;
    ink.set("normals", &normals)?;
    ink.set("outline.color", [0.08, 0.06, 0.1, 1.0])?;
    ink.set("outline.near", NEAR)?;
    ink.set("outline.far", FAR)?;
    ink.set("outline.width", 2.0 * scale)?;
    ink.set("outline.silhouette", 0.05)?;
    ink.set("outline.crease", 0.5)?;
    let outline = Pass::from_shader("outline", &ink);
    outline.load_previous();

    let scene = Scene::new();
    scene
        .add_pass(&normals_pass)
        .add_pass(&world)
        .add_pass(&outline);

    // One Shader draws the normals of every shape. The `mesh/transform`
    // registry snippet turns a normal by the model's transform.
    let normal = Shader::new(["mesh/transform", NORMALS].as_slice())?;

    let box_mesh = make_cube();
    let floor = ToonMaterial::new().base_color([0.78, 0.72, 0.62, 1.0])?;
    add_shape(
        &scene,
        &normal,
        &box_mesh,
        floor,
        [0.0, -0.2, 0.0],
        0.0,
        [7.0, 0.4, 7.0],
    )?;

    let block = ToonMaterial::new().base_color([0.3, 0.68, 0.74, 1.0])?;
    add_shape(
        &scene,
        &normal,
        &box_mesh,
        block,
        [1.6, 0.7, -0.8],
        0.6,
        [1.4; 3],
    )?;

    let ball = ToonMaterial::new()
        .base_color([0.95, 0.42, 0.33, 1.0])?
        .highlight([0.6, 0.6, 0.6])?;
    add_shape(
        &scene,
        &normal,
        &make_sphere(32, 16),
        ball,
        [-1.6, 1.0, -0.2],
        0.0,
        [1.0; 3],
    )?;

    let ring = ToonMaterial::new()
        .base_color([0.98, 0.78, 0.3, 1.0])?
        .highlight([0.5, 0.5, 0.5])?;
    add_shape(
        &scene,
        &normal,
        &make_torus(0.8, 0.3),
        ring,
        [0.4, 0.3, 1.8],
        0.0,
        [1.0; 3],
    )?;

    // Both passes that draw the shapes need the camera. Only `world` needs the light.
    let aspect = size[0] as f32 / size[1] as f32;
    let camera = Camera::perspective(0.8, aspect, NEAR, FAR)?;
    let sun = Light::directional([-0.5, -1.0, -0.4], [1.0, 0.96, 0.9])?.set_intensity(2.2)?;
    scene
        .add_to("normals", &camera)?
        .add_to("world", &camera)?
        .add_to("world", &sun)?;
    // The shadow side of each shape takes the ambient color, cooler than the sun.
    scene.ambient([0.3, 0.3, 0.4])?;

    Ok(Stage {
        scene,
        camera,
        depth,
        normals,
    })
}

/// Adds a shape twice over one mesh, at the same place: with `look` in the
/// `world` pass and with the `normal` shader in the `normals` pass.
fn add_shape(
    scene: &Scene,
    normal: &Shader,
    mesh: &Mesh,
    look: ToonMaterial,
    position: [f32; 3],
    turn: f32,
    size: [f32; 3],
) -> Result<(), AppError> {
    let model = Model::new(mesh.clone(), look);
    model.translate(position)?;
    model.rotate([0.0, 1.0, 0.0], turn)?;
    model.scale(size)?;
    let shape = Model::new(mesh.clone(), normal.clone());
    shape.set_transform(model.transform())?;
    scene.add_to("world", &model)?.add_to("normals", &shape)?;
    Ok(())
}

/// Puts the camera on a circle around the shapes, `angle` radians around it.
fn orbit(camera: &Camera, angle: f32) -> Result<(), AppError> {
    let eye = [angle.cos() * 10.0, 5.5, angle.sin() * 10.0];
    camera.look_at(eye, [0.0, 0.6, 0.0], [0.0, 1.0, 0.0])?;
    Ok(())
}

/// What the window callbacks read.
struct Viewer {
    stage: Stage,
    start: Instant,
}

/// Once the window exists: a target on it, and the stage at its size.
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let renderer = app.get_renderer();
    let window = windows.first().ok_or("no window")?;
    let target = renderer.create_target(window.clone()).await?;
    app.add_target(window.id(), target);

    let size = window.inner_size();
    let size = [size.width.max(1), size.height.max(1)];
    let stage = build_stage(renderer, size, window.scale_factor() as f32).await?;
    app.add_state(
        "viewer",
        Viewer {
            stage,
            start: Instant::now(),
        },
    );
    Ok(())
}

/// Each frame: move the camera a little further around, and draw.
fn draw(app: &App) {
    let id = app.primary_window_id();
    let Some(viewer) = app.get_state::<Viewer>("viewer") else {
        return;
    };
    let angle = viewer.start.elapsed().as_secs_f32() * 0.4;
    if let Err(error) = orbit(&viewer.stage.camera, angle) {
        eprintln!("orbit failed: {error}");
    }
    let renderer = app.get_renderer();
    let drawn = app.with_target(id, |target| renderer.render(&viewer.stage.scene, target));
    if let Some(Err(error)) = drawn {
        eprintln!("render failed: {error}");
    }
    if let Some(window) = app.window(id) {
        window.request_redraw();
    }
}

/// The depth and normals textures take the window's new size with it, since
/// the outline compares their pixels with the window's.
fn resize(app: &App, size: &PhysicalSize<u32>) {
    // A minimized window has no area, and keeps what it had.
    if size.width == 0 || size.height == 0 {
        return;
    }
    let new_size = [size.width, size.height];
    if let Err(error) = app.resize(new_size) {
        eprintln!("resize failed: {error}");
    }
    let Some(viewer) = app.get_state::<Viewer>("viewer") else {
        return;
    };
    let stage = &viewer.stage;
    let resized = stage
        .depth
        .resize(new_size)
        .and_then(|()| stage.normals.resize(new_size));
    if let Err(error) = resized {
        eprintln!("resize failed: {error}");
    }
    if let Err(error) = stage
        .camera
        .set_aspect(size.width as f32 / size.height as f32)
    {
        eprintln!("aspect not applied: {error}");
    }
}

/// Draws one frame into a texture and saves it as a PNG at `path`.
async fn save_still(path: &str) -> Result<(), AppError> {
    let size = [960, 640];
    let renderer = Renderer::new();
    let target = renderer.create_texture_target(size).await?;
    let stage = build_stage(&renderer, size, 1.0).await?;
    orbit(&stage.camera, 0.9)?;
    renderer.render(&stage.scene, &target)?;

    let pixels = target.get_image().await?;
    let image = image::RgbaImage::from_vec(size[0], size[1], pixels).ok_or("short readback")?;
    image.save(path)?;
    println!("saved {path}");
    Ok(())
}

fn main() -> Result<(), AppError> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if let [_, flag, path] = args.as_slice()
        && flag == "--png"
    {
        return pollster::block_on(save_still(path));
    }

    let mut app = App::new(Renderer::new());
    app.add_window(WindowAttributes::default().with_title("Toon shading with outlines"));
    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);
    app.run();
    Ok(())
}

/// A mesh from vertices given as a position and a normal, and the triangles
/// that index them, each counterclockwise seen from the side it faces.
fn build_mesh(vertices: Vec<([f32; 3], [f32; 3])>, indices: Vec<u32>) -> Mesh {
    let mesh = Mesh::new();
    mesh.add_vertices(
        vertices
            .into_iter()
            .map(|(position, normal)| Vertex::pbr(position).set(Vertex::NORMAL, normal)),
    );
    mesh.set_indices(indices);
    mesh
}

/// A cube one unit wide around the origin, with a flat normal on each face.
fn make_cube() -> Mesh {
    // Each face's normal, then two directions along it whose cross product is the normal.
    let faces: [[[f32; 3]; 3]; 6] = [
        [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],
        [[-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
        [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]],
        [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        [[0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    ];
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for [normal, across, up] in faces {
        let first = vertices.len() as u32;
        for [a, b] in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
            let corner = [0, 1, 2].map(|i| (normal[i] + across[i] * a + up[i] * b) * 0.5);
            vertices.push((corner, normal));
        }
        indices.extend([0, 1, 2, 0, 2, 3].map(|i| first + i));
    }
    build_mesh(vertices, indices)
}

/// A sphere of radius 1, in `rings` from pole to pole and `segments` around.
fn make_sphere(segments: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    for ring in 0..=rings {
        let polar = PI * ring as f32 / rings as f32;
        for segment in 0..=segments {
            let azimuth = TAU * segment as f32 / segments as f32;
            let normal = [
                polar.sin() * azimuth.cos(),
                polar.cos(),
                polar.sin() * azimuth.sin(),
            ];
            vertices.push((normal, normal));
        }
    }
    build_mesh(vertices, list_grid(segments, rings))
}

/// A ring lying flat around the vertical axis: `radius` to the middle of its
/// tube, whose own radius is `tube`.
fn make_torus(radius: f32, tube: f32) -> Mesh {
    let (around, across) = (48, 16);
    let mut vertices = Vec::new();
    for step in 0..=across {
        // The tube turns from the outside edge down first, so each triangle faces out.
        let turn = -TAU * step as f32 / across as f32;
        for segment in 0..=around {
            let angle = TAU * segment as f32 / around as f32;
            let normal = [
                turn.cos() * angle.cos(),
                turn.sin(),
                turn.cos() * angle.sin(),
            ];
            let middle = [radius * angle.cos(), 0.0, radius * angle.sin()];
            let position = [0, 1, 2].map(|i| middle[i] + normal[i] * tube);
            vertices.push((position, normal));
        }
    }
    build_mesh(vertices, list_grid(around, across))
}

/// The triangles of a grid of `columns` by `rows` quads, whose vertices run
/// along each row and repeat the first column at the end.
fn list_grid(columns: u32, rows: u32) -> Vec<u32> {
    let row = columns + 1;
    let mut indices = Vec::new();
    for y in 0..rows {
        for x in 0..columns {
            let a = y * row + x;
            let b = a + row;
            indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    indices
}
