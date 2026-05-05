// Step 2 — compose with the registry (JS port).
//
// Same Shader.fetch constructor as step 1, but with an array: a registry
// slug (noise/simplex2) followed by inline WGSL that calls it. The merged
// source is validated and linked into one program. The triangle still
// uses the `color` uniform; we multiply it by a soft simplex-noise breath
// so the surface shimmers.

import { Shader } from "fragmentcolor";

// #region: shader
const NOISY_TRIANGLE_WGSL = `
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p  = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.5),
        vec2<f32>( 0.6, -0.5),
        vec2<f32>( 0.0,  0.7),
    );
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0.0, 1.0);
    out.uv = (p[i] + vec2<f32>(1.0)) * 0.5;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let n = simplex2(in.uv * 6.0 + vec2<f32>(time * 0.4)) * 0.5 + 0.5;
    let breath = 0.55 + 0.45 * n;
    return vec4<f32>(color.rgb * breath, color.a);
}
`;
// #endregion: shader

// #region: setup
export async function setup(_renderer, _target) {
    // Shader.fetch is the async builder — slugs and URLs need fetching.
    const shader = await Shader.fetch(["noise/simplex2", NOISY_TRIANGLE_WGSL]);
    shader.set("color", [0.95, 0.30, 0.42, 1.0]);
    shader.set("time", 0.0);
    return { shader };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, _size) {
    const r = 0.5 + 0.45 * Math.sin(time * 0.7);
    const g = 0.5 + 0.45 * Math.cos(time * 0.5 + 1.7);
    const b = 0.5 + 0.45 * Math.sin(time * 0.9 + 3.1);
    state.shader.set("color", [r, g, b, 1.0]);
    state.shader.set("time", time);
    renderer.render(state.shader, target);
}
// #endregion: frame
