// Drawing with distance, step 2: a circle as a distance.

import { Shader } from "fragmentcolor";

const SHADER = `
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

// One triangle, big enough to cover the whole canvas.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(corners[i], 0.0, 1.0);
}

// #region: center
// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * resolution) / min(resolution.x, resolution.y);
    return vec2<f32>(p.x, -p.y);
}
// #endregion: center

// #region: show
// Magenta inside, azure outside, and a stripe every 0.05 units of distance.
fn show(d: f32) -> vec3<f32> {
    let tint = select(vec3<f32>(0.0, 0.51, 1.0), vec3<f32>(1.0, 0.0, 0.51), d < 0.0);
    let stripe = step(0.5, fract(d * 20.0));
    return tint * (0.3 + 0.45 * stripe);
}
// #endregion: show

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    let d = length(p) - 0.3;
    return vec4<f32>(show(d), 1.0);
}
// #endregion: paint
`;

export async function setup(renderer, target) {
    // #region: setup
    const shader = new Shader(SHADER);
    // #endregion: setup
    return { shader };
}

export function frame({ shader }, renderer, target, time, size) {
    // #region: frame
    shader.set("resolution", size);
    renderer.render(shader, target);
    // #endregion: frame
}
