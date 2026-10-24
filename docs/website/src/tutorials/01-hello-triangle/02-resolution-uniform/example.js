// Step 2: a second uniform, `resolution`, keeps the triangle's shape on any canvas.
// The page's <Demo> calls setup once and frame on every animation frame.

import { Shader } from "fragmentcolor";

const TRIANGLE = `
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
`;

// #region: setup
export function setup() {
    const shader = new Shader(TRIANGLE);
    shader.set("color", [0.95, 0.30, 0.42, 1.0]);
    return shader;
}
// #endregion: setup

// #region: frame
export function frame(shader, renderer, target, time) {
    const red = 0.5 + 0.45 * Math.sin(time * 0.7);
    const green = 0.5 + 0.45 * Math.cos(time * 0.5 + 1.7);
    const blue = 0.5 + 0.45 * Math.sin(time * 0.9 + 3.1);
    shader.set("color", [red, green, blue, 1.0]);

    const size = target.size();
    shader.set("resolution", [size.width, size.height]);
    renderer.render(shader, target);
}
// #endregion: frame
