// Step 5 — one triangle, many instances, a tiny particle field (JS port).
//
// Pulls `easing/in_out_sine` from the catalog so the per-instance pulse
// rides through a smoother bell-curve than a raw sine. The shader also
// carries the `resolution` uniform from step 2 so the field stays
// evenly distributed and each particle keeps its shape on any canvas.

import { Mesh, Shader, Vertex } from "fragmentcolor";

const PARTICLE_COUNT = 1500;

// #region: shader
const PARTICLE_WGSL = `
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,    // per-vertex
    @location(1) center: vec2<f32>,       // per-instance
    @location(2) phase: f32,               // per-instance
    @location(3) tint: vec3<f32>,          // per-instance
    @location(4) color: vec3<f32>,         // per-vertex
) -> VOut {
    let scale = 0.045;
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase) * 0.05,
        cos(time * 0.9 + phase * 1.4) * 0.05,
    );
    var world = position.xy * scale + center + wobble;
    // Aspect-correct so each particle's triangle stays equilateral and
    // the field stays evenly distributed regardless of canvas shape.
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    if (aspect > 1.0) { world.x = world.x / aspect; }
    else              { world.y = world.y * aspect; }

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

    // Same equilateral base triangle as step 4.
    const mesh = new Mesh();
    mesh.addVertices([
        new Vertex([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        new Vertex([ 0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        new Vertex([ 0.0,  0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);

    const TAU = Math.PI * 2;
    const instances = [];
    for (let i = 0; i < PARTICLE_COUNT; i++) {
        // Use a Vertex template so instance properties get auto-incrementing
        // locations starting at 1 — clear of the vertex `position` slot at
        // @location(0). The Vertex's position is dropped on conversion.
        instances.push(
            new Vertex([0.0, 0.0])
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
export function frame(state, renderer, target, time, size) {
    state.shader.set("resolution", size);
    state.shader.set("time", time);
    renderer.render(state.shader, target);
}
// #endregion: frame
