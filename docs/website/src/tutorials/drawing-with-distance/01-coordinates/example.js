// Drawing with distance, step 1: a coordinate for every pixel.

import { Shader } from "fragmentcolor";

const SHADER = `
// #region: triangle
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
// #endregion: triangle

// #region: center
// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * resolution) / min(resolution.x, resolution.y);
    return vec2<f32>(p.x, -p.y);
}
// #endregion: center

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    var color = vec3<f32>(0.5 + p.x, 0.5 + p.y, 0.5);
    if max(abs(p.x), abs(p.y)) > 0.5 {
        color = color * 0.3;
    }
    return vec4<f32>(color, 1.0);
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
