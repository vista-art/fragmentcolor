// Step 3 — a second uniform (JS port).
//
// The triangles in steps 1 and 2 squished as the canvas resized: NDC
// maps [-1, 1] across each axis independently, so positions hardcoded
// in clip space stretch with the canvas. The fix is information the
// shader doesn't have yet — the canvas aspect ratio. We add a second
// uniform, `resolution`, set it from the canvas pixel size each
// frame, and let the vertex stage aspect-correct the positions before
// output.

import { Shader } from "fragmentcolor";

// #region: shader
const RESOLUTION_TRIANGLE_WGSL = `
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.5),
        vec2<f32>( 0.6, -0.5),
        vec2<f32>( 0.0,  0.7),
    );
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var pos = p[i];
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
`;
// #endregion: shader

// #region: setup
export async function setup(_renderer, _target) {
    const shader = new Shader(RESOLUTION_TRIANGLE_WGSL);
    shader.set("color", [0.95, 0.30, 0.42, 1.0]);
    return { shader };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, size) {
    const r = 0.5 + 0.45 * Math.sin(time * 0.7);
    const g = 0.5 + 0.45 * Math.cos(time * 0.5 + 1.7);
    const b = 0.5 + 0.45 * Math.sin(time * 0.9 + 3.1);
    state.shader.set("color", [r, g, b, 1.0]);
    // The new uniform is updated from the canvas pixel size every
    // frame, so resizing the page keeps the triangle proportions
    // correct.
    state.shader.set("resolution", size);
    renderer.render(state.shader, target);
}
// #endregion: frame
