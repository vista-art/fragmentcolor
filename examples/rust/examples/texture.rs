use fragmentcolor::{App, Shader, run};
use std::path::PathBuf;

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

pub fn on_resize(app: &App, new_size: &winit::dpi::PhysicalSize<u32>) {
    app.resize([new_size.width, new_size.height]);
}

fn main() {
    // Load a small built-in asset (use favicon from docs/website/public if present)
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // examples/rust
    path.push("docs/website/public/favicon.png");

    let shader = Shader::new(VS_FS).unwrap();

    // Create a headless renderer/target via App convenience
    let mut app = App::new();

    // Create texture using the same renderer instance held by App and set it on the shader
    if path.exists() {
        let tex = pollster::block_on(app.renderer().create_texture(&path)).unwrap();
        shader.set("tex", &tex).unwrap();
    }

    app.scene(shader).on_resize(on_resize);

    run(&mut app);
}
