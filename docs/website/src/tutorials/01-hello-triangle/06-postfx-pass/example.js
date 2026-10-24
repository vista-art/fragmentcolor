// Step 6: the particles render into a texture, and a second pass grades it for the screen.
// Call setup once, then frame on every animation frame, such as from requestAnimationFrame.

import { Instance, Mesh, Pass, Shader, Vertex } from "fragmentcolor";

const COPIES = 15000;

const PARTICLES = `
// #region: particle-shader
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
    var shape = position.xy * 0.03;
    if (aspect > 1.0) { shape.x = shape.x / aspect; }
    else              { shape.y = shape.y * aspect; }

    // Each copy loops around its center, out of step with the others.
    let wobble = vec2<f32>(
        sin(time * 1.3 + phase),
        cos(time * 0.9 + phase * 1.4),
    ) * 0.03;

    // in_out_sine comes from easing/in_out_sine and flattens the wave near 0 and 1.
    let wave = 0.5 + 0.5 * sin(time * 2.0 + phase);
    let glow = 0.75 + 0.75 * in_out_sine(wave);

    var out: VOut;
    out.pos = vec4<f32>(center + wobble + shape, 0.0, 1.0);
    out.color = color * tint * glow;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: particle-shader
`;

const POSTFX = `
// #region: postfx-shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> time: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // One triangle large enough to cover the whole screen.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0),
    );
    var out: VOut;
    out.pos = vec4<f32>(corners[i], 0.0, 1.0);
    out.uv = uvs[i];
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    // Read red, green and blue from three slightly different spots.
    let offsets = chromatic_offsets(in.uv, 0.006);
    let red = textureSample(scene, scene_sampler, offsets[0]).r;
    let green = textureSample(scene, scene_sampler, offsets[1]).g;
    let blue = textureSample(scene, scene_sampler, offsets[2]).b;

    // Expose at half brightness, then roll off the highlights like film.
    let graded = tonemap_filmic(vec3<f32>(red, green, blue) * 0.5);
    let edges = vignette(in.uv, 0.55, 0.40);
    let grain = film_grain(in.uv, time) * 0.05;
    return vec4<f32>(graded * edges + grain, 1.0);
}
// #endregion: postfx-shader
`;

// #region: setup
export async function setup(renderer) {
    const particles = await Shader.fetch(["easing/in_out_sine", PARTICLES]);

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
            0.2 + Math.random() * 0.8,
            0.2 + Math.random() * 0.8,
            0.2 + Math.random() * 0.8,
        ];
        mesh.addInstance(
            new Instance()
                .set("center", center)
                .set("phase", phase)
                .set("tint", tint),
        );
    }
    particles.addMesh(mesh);

    // The particles draw into this texture, and the postfx shader reads it.
    const scene = await renderer.createTextureTarget([1024, 1024]);

    const postfx = await Shader.fetch([
        "postfx/chromatic_offsets",
        "color/tonemap_filmic",
        "postfx/vignette",
        "postfx/film_grain",
        POSTFX,
    ]);
    postfx.set("scene", scene.texture());

    const particlePass = Pass.fromShader("particles", particles);
    particlePass.setTarget(scene);
    particlePass.setClearColor([0.04, 0.05, 0.08, 1.0]);

    const output = Pass.fromShader("postfx", postfx);
    output.require(particlePass);

    return { particles, postfx, output };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time) {
    const size = target.size();
    state.particles.set("resolution", [size.width, size.height]);
    state.particles.set("time", time);
    state.postfx.set("time", time);
    renderer.render(state.output, target);
}
// #endregion: frame
