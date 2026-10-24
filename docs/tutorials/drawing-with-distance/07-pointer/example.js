// Drawing with distance, step 7: the pointer, and knobs.

import { Shader } from "fragmentcolor";

const SHADER = `
// The star and box distances, the smooth union and the contour coloring
// follow Inigo Quilez: https://iquilezles.org/articles/distfunctions2d/

// #region: params
struct Params {
    resolution: vec2<f32>,
    pointer: vec2<f32>,
    time: f32,
    blend: f32, // 0..0.4
    bands: f32, // 5..120
    inside_color: vec3<f32>,
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

    var color = select(u.outside_color, u.inside_color, d < 0.0);
    color = color * (1.0 - 0.75 * exp(-8.0 * abs(d)));
    color = color * (0.85 + 0.15 * cos(6.2831853 * (u.bands * d - 0.5 * t)));
    let pixel_size = fwidth(d);
    color = mix(color, vec3<f32>(1.0), 1.0 - smoothstep(pixel_size, 2.0 * pixel_size, abs(d)));
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
    shader.set("u.bands", 40);
    shader.set("u.inside_color", [0.38, 0.62, 0.92]);
    shader.set("u.outside_color", [0.92, 0.6, 0.32]);
    // #endregion: setup
    const size = target.size();
    shader.set("u.pointer", [size.width / 2, size.height / 2]);

    // #region: pointer
    addEventListener("pointermove", (event) => {
        const canvas = event.composedPath().find((node) => node instanceof HTMLCanvasElement);
        if (!canvas) return;
        const box = canvas.getBoundingClientRect();
        shader.set("u.pointer", [
            ((event.clientX - box.left) / box.width) * canvas.width,
            ((event.clientY - box.top) / box.height) * canvas.height,
        ]);
    });
    // #endregion: pointer
    return { shader };
}

export function frame({ shader }, renderer, target, time, size) {
    // #region: frame
    shader.set("u.resolution", size);
    shader.set("u.time", time);
    renderer.render(shader, target);
    // #endregion: frame
}
