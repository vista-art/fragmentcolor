// Step 4 — one triangle, many instances, a tiny particle field (JS port).
//
// Pulls `easing/in_out_sine` from the catalog so the per-instance pulse
// rides through a smoother bell-curve than a raw sine.

import { Instance, Mesh, Shader, Vertex } from "fragmentcolor";

const PARTICLE_COUNT = 1500;

// #region: shader
const PARTICLE_WGSL = `
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) center: vec2<f32>,
    @location(3) phase: f32,
    @location(4) tint: vec3<f32>,
) -> VOut {
    let scale = 0.045;
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase) * 0.05,
        cos(time * 0.9 + phase * 1.4) * 0.05,
    );
    let world = position.xy * scale + center + wobble;
    // Same oscillation as before, routed through \`easing/in_out_sine\`
    // (a registry slug we pulled in at Shader.fetch) for a softer pulse.
    let raw = 0.5 + 0.5 * sin(time * 2.0 + phase);
    let glow = 0.4 + 0.6 * in_out_sine(raw);

    var out: VOut;
    out.pos = vec4<f32>(world, 0.0, 1.0);
    out.color = color * tint * glow;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
`;
// #endregion: shader

// #region: setup
export async function setup(_renderer, _target) {
    // Shader.fetch is the async builder — needed because slugs and URLs
    // require fetching. Same constructor as the rest of the tutorial,
    // just with one more entry in the array.
    const shader = await Shader.fetch(["easing/in_out_sine", PARTICLE_WGSL]);
    shader.set("time", 0.0);

    const mesh = new Mesh();
    mesh.addVertices([
        new Vertex([-0.6, -0.5, 0.0]).set("color", [0.95, 0.30, 0.42]),
        new Vertex([ 0.6, -0.5, 0.0]).set("color", [0.30, 0.85, 0.55]),
        new Vertex([ 0.0,  0.7, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);

    const TAU = Math.PI * 2;
    const instances = [];
    for (let i = 0; i < PARTICLE_COUNT; i++) {
        instances.push(
            new Instance()
                .set("center", [Math.random() * 1.8 - 0.9, Math.random() * 1.8 - 0.9])
                .set("phase", Math.random() * TAU)
                .set("tint", [
                    0.6 + Math.random() * 0.4,
                    0.6 + Math.random() * 0.4,
                    0.6 + Math.random() * 0.4,
                ]),
        );
    }
    mesh.addInstances(instances);

    shader.addMesh(mesh);
    return { shader, mesh };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, _size) {
    state.shader.set("time", time);
    renderer.render(state.shader, target);
}
// #endregion: frame
