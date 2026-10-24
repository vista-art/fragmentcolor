// Drawing with distance, step 7: the pointer, and knobs.

import { Shader } from "fragmentcolor";

const SHADER = `
// The star and box distances and the smooth union
// follow Inigo Quilez: https://iquilezles.org/articles/distfunctions2d/

// #region: params
struct Params {
    resolution: vec2<f32>,
    pointer: vec2<f32>,
    time: f32,
    blend: f32, // 0..0.4
    rings: f32, // 5..60
    glow: f32, // 2..30
    inside_color: vec3<f32>,
    edge_color: vec3<f32>,
    outside_color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Params;
// #endregion: params

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

// Moves the origin to the middle of the canvas,
// turns y to point up, and scales the shorter side to one unit.
fn center(pixel: vec2<f32>) -> vec2<f32> {
    let p = (pixel - 0.5 * u.resolution) / min(u.resolution.x, u.resolution.y);
    return vec2<f32>(p.x, -p.y);
}

// #region: paint
@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = center(pixel.xy);
    let pointer = center(u.pointer);
    let t = u.time;

    let d_star = star(rotate2(p - vec2<f32>(-0.25, 0.0), t * 0.4), 0.2, 5u, 3.0);
    let d_box = box(p - vec2<f32>(0.25, 0.05 * sin(t)), vec2<f32>(0.13));
    let d_dot = circle(p - pointer, 0.08);
    let d = op_smooth_union(op_smooth_union(d_star, d_box, u.blend), d_dot, u.blend);

    let outside = max(d, 0.0);
    let pixel_size = fwidth(d);
    var color = vec3<f32>(0.05, 0.06, 0.08) + u.outside_color * 0.45 * exp(-u.glow * outside);
    let ring = abs(fract(d * u.rings - 0.5 * t + 0.5) - 0.5) / (u.rings * pixel_size);
    color = color + u.outside_color * 0.35 * exp(-3.0 * outside) * (1.0 - smoothstep(0.5, 1.5, ring));
    color = select(color, mix(u.edge_color, u.inside_color, smoothstep(0.0, 0.12, -d)), d < 0.0);
    color = mix(color, vec3<f32>(1.0), 1.0 - smoothstep(0.5 * pixel_size, 1.5 * pixel_size, abs(d)));
    return vec4<f32>(color, 1.0);
}
// #endregion: paint
`;

export async function setup(renderer, target) {
    // #region: setup
    const shader = await Shader.fetch([
        "sdf2d/circle",
        "sdf2d/box",
        "sdf2d/star",
        "sdf/op_smooth_union",
        "map/rotate2",
        SHADER,
    ]);
    shader.set("u.blend", 0.12);
    shader.set("u.rings", 20);
    shader.set("u.glow", 10);
    shader.set("u.inside_color", [1, 0, 0.51]);
    shader.set("u.edge_color", [0.51, 0, 1]);
    shader.set("u.outside_color", [0, 0.51, 1]);
    // #endregion: setup
    return { shader };
}

export function frame({ shader }, renderer, target, time, size, pointer) {
    // #region: frame
    shader.set("u.resolution", size);
    shader.set("u.time", time);
    shader.set("u.pointer", pointer);
    renderer.render(shader, target);
    // #endregion: frame
}
