// Step 4: the program owns the triangle's corners, each with its own color.
// The page's <Demo> calls setup once and frame on every animation frame.

import { Mesh, Shader, Vertex } from "fragmentcolor";

const GRADIENT = `
// #region: shader
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
    let aspect = resolution.x / resolution.y;
    var pos = position.xy;
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: shader
`;

// #region: setup
export function setup() {
    const shader = new Shader(GRADIENT);

    const mesh = new Mesh();
    mesh.addVertices([
        new Vertex([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        new Vertex([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        new Vertex([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ]);
    shader.addMesh(mesh);
    return shader;
}
// #endregion: setup

// #region: frame
export function frame(shader, renderer, target, _time) {
    const size = target.size();
    shader.set("resolution", [size.width, size.height]);
    renderer.render(shader, target);
}
// #endregion: frame
