use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, call, run};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// Inline WGSL for a fullscreen triangle with solid color output
const FS_TRIANGLE: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(
    vec2<f32>(-1.,-1.),
    vec2<f32>( 3.,-1.),
    vec2<f32>(-1., 3.)
  );
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@group(0) @binding(0) var<uniform> color: vec4<f32>;
@fragment
fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return color; }
"#;

fn on_resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

fn draw(app: &App) {
    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    let shader = Shader::new(FS_TRIANGLE)?;
    shader.set("color", [1.0, 0.0, 0.0, 1.0])?; // red
    let pass = Pass::from_shader("fullscreen", &shader);

    let mut frame = Frame::new();
    frame.add_pass(&pass);
    app.add("frame.main", frame);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_start(call!(setup))
        .on_resize(on_resize)
        .on_redraw_requested(draw);
    run(&mut app);
    Ok(())
}
