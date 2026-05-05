// Step 2 — your vertices, your colours.
//
// In step 1 the triangle's positions and colour were locked inside the
// shader. Here we hand control back to the program: three Vertex objects
// each carry a position *and* a colour, the shader interpolates colour
// across the surface, and we get a smooth gradient for free.

use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Renderer, SetupResult, Shader, call, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// #region: shader
const GRADIENT_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;
// #endregion: shader

// #region: setup
async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = Shader::new(GRADIENT_WGSL)?;

    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.6, -0.5, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex::new([0.6, -0.5, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex::new([0.0, 0.7, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);
    shader.add_mesh(&mesh)?;

    app.add("shader.main", shader);
    app.add("mesh.main", mesh);

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
    if let Some(shader) = app.get::<Shader>("shader.main") {
        let id = app.primary_window_id();
        let renderer = app.get_renderer();
        let _ = app.with_target(id, |target| renderer.render(&*shader, target));
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
