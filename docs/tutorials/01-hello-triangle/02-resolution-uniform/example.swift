// Step 2: a second uniform, `resolution`, keeps the triangle's shape on any screen.
// Call setup once, then frame on every display refresh, such as from an MTKView delegate.

import FragmentColor
import Foundation

let triangle = """
// #region: shader
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.4),
        vec2<f32>( 0.7, -0.4),
        vec2<f32>( 0.0,  0.8),
    );
    // Shrink the longer axis so one unit spans the same pixels on both.
    let aspect = resolution.x / resolution.y;
    var pos = corners[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
// #endregion: shader
"""

// #region: setup
func setup() throws -> Shader {
    let shader = try Shader(triangle)
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

    let size = target.size()
    try shader.set("resolution", [Float(size.width), Float(size.height)])
    try renderer.render(shader, target)
}
// #endregion: frame
