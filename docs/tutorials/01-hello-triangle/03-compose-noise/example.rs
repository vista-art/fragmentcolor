//! Step 3: `simplex2` from the shader library makes the color shimmer.

#[path = "../host.rs"]
mod host;

use fragmentcolor::{FragmentColorError, Renderer, Shader, Target};

const TRIANGLE: &str = r#"
// #region: shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.404),
        vec2<f32>( 0.7, -0.404),
        vec2<f32>( 0.0,  0.808),
    );
    let aspect = resolution.x / resolution.y;
    var pos = corners[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    // The corner before the aspect fix, moved from -1..1 into 0..1.
    out.uv = (corners[i] + vec2<f32>(1.0)) * 0.5;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    // simplex2 comes from noise/simplex2 and returns roughly -1..1.
    let noise = simplex2(in.uv * 6.0 + vec2<f32>(time * 0.4));
    let brightness = 0.55 + 0.45 * (noise * 0.5 + 0.5);
    return vec4<f32>(color.rgb * brightness, color.a);
}
// #endregion: shader
"#;

// #region: setup
fn setup() -> Result<Shader, FragmentColorError> {
    let shader = Shader::new(["noise/simplex2", TRIANGLE])?;
    shader.set("color", [0.95, 0.30, 0.42, 1.0])?;
    Ok(shader)
}
// #endregion: setup

// #region: frame
fn frame(
    shader: &Shader,
    renderer: &Renderer,
    target: &impl Target,
    time: f32,
) -> Result<(), FragmentColorError> {
    let red = 0.5 + 0.45 * (time * 0.7).sin();
    let green = 0.5 + 0.45 * (time * 0.5 + 1.7).cos();
    let blue = 0.5 + 0.45 * (time * 0.9 + 3.1).sin();
    shader.set("color", [red, green, blue, 1.0])?;
    shader.set("time", time)?;

    let size = target.size();
    shader.set("resolution", [size.width as f32, size.height as f32])?;
    renderer.render(shader, target)?;
    Ok(())
}
// #endregion: frame

fn main() {
    host::run(async |_: &Renderer| setup(), frame);
}
