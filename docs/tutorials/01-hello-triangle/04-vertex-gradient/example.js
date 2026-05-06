// Step 4 — your vertices, your colors (JS port).

import { Mesh, Shader, Vertex } from "fragmentcolor";

// #region: shader
const GRADIENT_WGSL = `
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VOut {
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var p = position;
    if (aspect > 1.0) { p.x = p.x / aspect; }
    else              { p.y = p.y * aspect; }
    var out: VOut;
    out.pos = vec4<f32>(p, 1.0);
    out.color = color;
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
    const shader = new Shader(GRADIENT_WGSL);

    // Same equilateral positions as the earlier steps; one color per corner.
    const mesh = new Mesh();
    mesh.addVertices([
        new Vertex([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        new Vertex([ 0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        new Vertex([ 0.0,  0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);
    shader.addMesh(mesh);

    return { shader, mesh };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, _time, size) {
    state.shader.set("resolution", size);
    renderer.render(state.shader, target);
}
// #endregion: frame
