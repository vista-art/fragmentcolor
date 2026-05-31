// Step 4 — your vertices, your colors.
//
// In steps 1 to 3 the triangle's positions were locked inside the
// shader. Here we hand control back to the program: three Vertex
// objects each carry a position *and* a color, the shader
// interpolates the color across the surface, and we get a smooth
// gradient for free. The `resolution` uniform from step 2 carries
// over so the gradient triangle keeps its shape on any canvas.

use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{App, Renderer, SetupResult, Shader, Target, call, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// #region: shader
const GRADIENT_WGSL: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VOut {
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var p = position;
    if (aspect > 1.0) { p.x = p.x / aspect; }
    else              { p.y = p.y * aspect; }
    var out: VOut;
    out.pos = vec4<f32>(p, 1.0);
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

    // Same equilateral positions as the earlier steps; one color per corner.
    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex::new([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex::new([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
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
        let _ = app.with_target(id, |target| {
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
