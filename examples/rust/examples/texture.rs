use fragmentcolor::{App, Frame, Pass, Renderer, SetupResult, Shader, call, run};
use std::path::PathBuf;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const VS_FS: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
"#;

pub fn resize(app: &App, new_size: &PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

async fn setup(app: &App, windows: Vec<Arc<Window>>) -> SetupResult {
    // Load a small built-in asset (use favicon from docs/website/public)
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // examples/rust
    path.push("docs/website/public/favicon.png");

    let shader = Shader::new(VS_FS)?;
    if path.exists() {
        let tex = app.get_renderer().create_texture(&path).await?;
        shader.set("tex", &tex)?;
    }
    let pass = Pass::from_shader("main", &shader);
    let mut frame = Frame::new();
    frame.add_pass(&pass);
    app.add("frame.main", frame);

    for win in windows {
        let target = app.get_renderer().create_target(win.clone()).await?;
        app.add_target(win.id(), target);
    }
    Ok(())
}

fn draw(app: &App) {
    let id = app.primary_window_id();
    if let Some(frame) = app.get::<Frame>("frame.main") {
        let r = app.get_renderer();
        let _ = app.with_target(id, |t| r.render(&*frame, t));
    }
}

fn main() {
    let renderer = Renderer::new();
    let mut app = App::new(renderer);
    app.on_start(call!(setup))
        .on_resize(resize)
        .on_redraw_requested(draw);
    run(&mut app);
}
