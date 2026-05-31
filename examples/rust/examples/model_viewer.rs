//! Windowed "Hello glTF" model viewer.
//!
//! Loads a `Scene` from glTF via `Scene::load`, opens a window with the `App`
//! helper, and orbits a `Camera` around the model so you can see it from every
//! side. This is the runnable counterpart to the orbit-camera snippet in the
//! "Building an App in Rust" guide.
//!
//! The asset here is a tiny in-memory `.glb` (one triangle) so the example
//! runs with no external files. To view a real model, swap the `SceneSource`
//! for a path: `Scene::load(SceneSource::gltf("path/to/model.glb"))`.

use fragmentcolor::{App, Camera, Renderer, Scene, SceneSource, SetupResult, call};
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::{Window, WindowAttributes};

/// Everything the per-frame `draw` callback needs to read. Stashed on the
/// App's typed state registry by `main`, fetched back in `draw` / `resize`.
struct Viewer {
    scene: Scene,
    camera: Camera,
    start: Instant,
}

/// One-time setup once winit hands us a real window: wire a render target to
/// it. `Scene::load` already ran in `main` (it's sync and needs no renderer),
/// so the only async work left is acquiring the window surface.
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}

/// Per-frame tick (fires on `RedrawRequested`): advance the orbit, render the
/// scene to the window's target, then request the next frame.
fn draw(app: &App) {
    let id = app.primary_window_id();
    let Some(viewer) = app.get_state::<Viewer>("viewer") else {
        return; // setup hasn't run yet
    };

    // Orbit the camera around the origin — one full turn every ~6.3 seconds.
    let t = viewer.start.elapsed().as_secs_f32();
    let radius = 3.0_f32;
    viewer.camera.look_at(
        [t.cos() * radius, 1.0, t.sin() * radius],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );

    let renderer = app.get_renderer();
    let _ = app.with_target(id, |target| renderer.render(&viewer.scene, target));

    if let Some(win) = app.window(id) {
        win.request_redraw();
    }
}

/// Keep the projection square when the window is resized. Same Arc-shared
/// camera backing — no Scene rebuild.
fn resize(app: &App, size: &PhysicalSize<u32>) {
    app.resize([size.width, size.height]);
    if let Some(viewer) = app.get_state::<Viewer>("viewer") {
        let aspect = size.width as f32 / size.height.max(1) as f32;
        viewer.camera.set_aspect(aspect);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.add_window(WindowAttributes::default().with_title("Hello glTF — model viewer"));

    // Build the scene up front: `Scene::load` is synchronous and renderer-free,
    // so it doesn't need to wait for the window. The Scene injects a default
    // light at render time, so the model reads as lit without us adding one.
    let scene = Scene::load(SceneSource::gltf(build_minimal_triangle_glb())).expect("Scene::load");
    let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
        [0.0, 1.0, 3.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    scene.add(&camera).expect("scene.add(camera)");
    app.add_state(
        "viewer",
        Viewer {
            scene,
            camera,
            start: Instant::now(),
        },
    );

    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);
    app.run();
    Ok(())
}

/// Hand-build a valid `.glb` payload in memory: one triangle, positions only.
/// `Scene::load`'s face-normal fallback fills in the missing normals so the
/// PBR shader lights it. In real use you'd hand `Scene::load` a file path or
/// `.glb` bytes instead.
fn build_minimal_triangle_glb() -> Vec<u8> {
    #[rustfmt::skip]
    let positions: [f32; 9] = [
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];
    let bin: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    let bin_len = bin.len() as u32;

    let json = r#"{"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"mode":4}]}],"buffers":[{"byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36,"byteOffset":0}],"accessors":[{"bufferView":0,"byteOffset":0,"componentType":5126,"count":3,"type":"VEC3","min":[-0.5,-0.5,0.0],"max":[0.5,0.5,0.0]}],"asset":{"version":"2.0"}}"#;
    let mut json_bytes = json.as_bytes().to_vec();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    let json_len = json_bytes.len() as u32;
    let total = 12 + 8 + json_len + 8 + bin_len;

    let mut glb = Vec::with_capacity(total as usize);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&total.to_le_bytes());
    glb.extend_from_slice(&json_len.to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&bin_len.to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);
    glb
}
