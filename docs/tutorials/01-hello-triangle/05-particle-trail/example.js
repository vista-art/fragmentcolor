// Step 5: one mesh drawn 15000 times, each copy with its own place, phase and tint.
// The page's <Demo> calls setup once and frame on every animation frame.

import { Instance, Mesh, Shader, Vertex } from "fragmentcolor";

const COPIES = 15000;

const PARTICLES = `
// #region: shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>, // per vertex
    @location(1) color: vec3<f32>,    // per vertex
    @location(2) center: vec2<f32>,   // per instance
    @location(3) phase: f32,          // per instance
    @location(4) tint: vec3<f32>,     // per instance
) -> VOut {
    // Shrink the triangle and keep its shape, as in step 2.
    let aspect = resolution.x / resolution.y;
    var shape = position.xy * 0.018;
    if (aspect > 1.0) { shape.x = shape.x / aspect; }
    else              { shape.y = shape.y * aspect; }

    // Each copy circles its center on its own schedule.
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase),
        cos(time * 0.9 + phase * 1.4),
    ) * 0.03;

    // in_out_sine comes from easing/in_out_sine. It eases a 0..1 wave
    // so each triangle lingers at its brightest and its dimmest.
    let wave = 0.5 + 0.5 * sin(time * 2.0 + phase);
    let glow = 0.5 + 0.5 * in_out_sine(wave);

    var out: VOut;
    out.pos = vec4<f32>(center + wobble + shape, 0.0, 1.0);
    out.color = color * tint * glow;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: shader
`;

// #region: setup
export async function setup() {
    const shader = await Shader.fetch(["easing/in_out_sine", PARTICLES]);

    const mesh = new Mesh();
    mesh.addVertices([
        new Vertex([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        new Vertex([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        new Vertex([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);

    for (let copy = 0; copy < COPIES; copy++) {
        const center = [Math.random() * 2 - 1, Math.random() * 2 - 1];
        const phase = Math.random() * 2 * Math.PI;
        const tint = [
            0.6 + Math.random() * 0.4,
            0.6 + Math.random() * 0.4,
            0.6 + Math.random() * 0.4,
        ];
        mesh.addInstance(
            new Instance()
                .set("center", center)
                .set("phase", phase)
                .set("tint", tint),
        );
    }

    shader.addMesh(mesh);
    return shader;
}
// #endregion: setup

// #region: frame
export function frame(shader, renderer, target, time) {
    const size = target.size();
    shader.set("resolution", [size.width, size.height]);
    shader.set("time", time);
    renderer.render(shader, target);
}
// #endregion: frame
