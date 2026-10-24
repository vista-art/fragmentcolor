//! Step 4: the program owns the triangle's corners, each with its own color.

#[path = "../host.rs"]
mod host;

use fragmentcolor::{FragmentColorError, Mesh, Renderer, Shader, Target, Vertex};

const GRADIENT: &str = r#"
// #region: shader
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
    let aspect = resolution.x / resolution.y;
    var pos = position.xy;
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: shader
"#;

// #region: setup
fn setup() -> Result<Shader, FragmentColorError> {
    let shader = Shader::new(GRADIENT)?;

    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex::new([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex::new([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);
    shader.add_mesh(&mesh)?;
    Ok(shader)
}
// #endregion: setup

// #region: frame
fn frame(
    shader: &Shader,
    renderer: &Renderer,
    target: &impl Target,
    _time: f32,
) -> Result<(), FragmentColorError> {
    let size = target.size();
    shader.set("resolution", [size.width as f32, size.height as f32])?;
    renderer.render(shader, target)?;
    Ok(())
}
// #endregion: frame

fn main() {
    host::run(async |_: &Renderer| setup(), frame);
}
