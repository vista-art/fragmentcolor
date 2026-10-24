// Step 3: `simplex2` from the shader library makes the color shimmer.
// Call `setup` once, then call `frame` on every display refresh, such as from an `MTKView` delegate.

import FragmentColor
import Foundation

let triangle = """
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
"""

// #region: setup
func setup() throws -> Shader {
    let shader = try Shader(["noise/simplex2", triangle])
    try shader.set("color", [0.95, 0.30, 0.42, 1.0])
    return shader
}
// #endregion: setup

// #region: frame
func frame(_ shader: Shader, _ renderer: Renderer, _ target: WindowTarget, _ time: Float) throws {
    let red = 0.5 + 0.45 * sin(time * 0.7)
    let green = 0.5 + 0.45 * cos(time * 0.5 + 1.7)
    let blue = 0.5 + 0.45 * sin(time * 0.9 + 3.1)
    try shader.set("color", [red, green, blue, 1.0])
    try shader.set("time", time)

    let size = target.size()
    try shader.set("resolution", [Float(size.width), Float(size.height)])
    try renderer.render(shader, target)
}
// #endregion: frame
