// Step 1: a shader with one uniform, drawn every frame in a new color.
// Call setup once, then frame on every animation frame, such as from requestAnimationFrame.

import { Shader } from "fragmentcolor";

export const TRIANGLE = `
// #region: shader
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Three corners of an equilateral triangle, centered on the origin.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.4),
        vec2<f32>( 0.7, -0.4),
        vec2<f32>( 0.0,  0.8),
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
    renderer.render(shader, target);
}
// #endregion: frame
