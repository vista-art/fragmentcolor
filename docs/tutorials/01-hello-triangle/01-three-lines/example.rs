//! Step 1: a shader with one uniform draws the triangle in a new color every frame.

#[path = "../host.rs"]
mod host;

use fragmentcolor::{FragmentColorError, Renderer, Shader, Target};

const TRIANGLE: &str = r#"
// #region: shader
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Three corners of an equilateral triangle, centered on the origin.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.404),
        vec2<f32>( 0.7, -0.404),
        vec2<f32>( 0.0,  0.808),
    );
    var out: VOut;
    out.pos = vec4<f32>(corners[i], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
// #endregion: shader
"#;

// #region: setup
fn setup() -> Result<Shader, FragmentColorError> {
    let shader = Shader::new(TRIANGLE)?;
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
    renderer.render(shader, target)?;
    Ok(())
}
// #endregion: frame

fn main() {
    host::run(async |_: &Renderer| setup(), frame);
}
