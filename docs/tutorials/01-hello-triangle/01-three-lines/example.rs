// Step 1 — load, set, render.
//
// The smallest meaningful FragmentColor pipeline: a Renderer creates a
// Target wrapped around a window, and a Shader fetched from a URL exposes
// one settable uniform. We cycle the `color` uniform every frame to show
// that the same shader handle is the surface for live updates — no
// rebuild, no rebind.
//
// Outside the `setup` and `frame` regions below is the tiny winit
// bootstrap that opens a window, drives the redraw loop, and blocks on
// the OS event loop. It's identical in every step of this tutorial; the
// FragmentColor API surface lives entirely inside the two regions.

use fragmentcolor::{App, Renderer, SetupResult, Shader, Target, call, run};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = Shader::new("https://fragmentcolor.org/triangle.wgsl")?;
    shader.set("color", [0.95, 0.30, 0.42, 1.0])?;
    app.add("shader.main", shader);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}
// #endregion: setup

fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

// #region: frame
fn draw(app: &App) {
    static START: OnceLock<Instant> = OnceLock::new();
    let time = START.get_or_init(Instant::now).elapsed().as_secs_f32();

    if let Some(shader) = app.get::<Shader>("shader.main") {
        let r = 0.5 + 0.45 * (time * 0.7).sin();
        let g = 0.5 + 0.45 * (time * 0.5 + 1.7).cos();
        let b = 0.5 + 0.45 * (time * 0.9 + 3.1).sin();
        let _ = shader.set("color", [r, g, b, 1.0]);

        let id = app.primary_window_id();
        let renderer = app.get_renderer();
        let _ = app.with_target(id, |target| {
            // The triangle.wgsl shader takes a `resolution` uniform so it
            // can aspect-correct its baked positions on any window size.
            let s = target.size();
            let _ = shader.set("resolution", [s.width as f32, s.height as f32]);
            renderer.render(&*shader, target)
        });
    }
}
// #endregion: frame

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);
    run(&mut app);
    Ok(())
}
