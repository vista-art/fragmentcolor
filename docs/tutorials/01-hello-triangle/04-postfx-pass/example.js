// Step 4 — postfx, the right way (JS port).
//
// Same particles as step 3, but they pass through an intermediate
// texture and then a postfx shader composed from two registry slugs.

import { Instance, Mesh, Pass, Shader, Vertex } from "fragmentcolor";

const PARTICLE_COUNT = 1500;
const SCENE_SIZE = 1024;

// #region: particle-shader
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
    let glow = 0.55 + 0.45 * sin(time * 2.0 + phase);

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
// #endregion: particle-shader

// #region: postfx-shader
const POSTFX_MAIN_WGSL = `
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p  = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0., 1.), vec2<f32>(2., 1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> time: f32;

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let base   = textureSample(scene, samp, in.uv).rgb;
    let v_mask = vignette(in.uv, 0.55, 0.40);
    let g      = film_grain(in.uv, time) * 0.06;
    return vec4<f32>(base * v_mask + g, 1.0);
}
`;
// #endregion: postfx-shader

// #region: setup
export async function setup(renderer, _target) {
    // Particle shader and mesh — same as step 3.
    const particleShader = new Shader(PARTICLE_WGSL);
    particleShader.set("time", 0.0);

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

    // Intermediate texture target — the bridge between the two passes.
    const intermediate = await renderer.createTextureTarget([SCENE_SIZE, SCENE_SIZE]);
    const sceneTexture = intermediate.texture();

    // Postfx shader composed from two registry slugs and a main source.
    // Shader.fetch is the async builder — needed because slugs/URLs require fetching.
    const postfxShader = await Shader.fetch([
        "postfx/vignette",
        "postfx/film_grain",
        POSTFX_MAIN_WGSL,
    ]);
    postfxShader.set("scene", sceneTexture);
    postfxShader.set("time", 0.0);

    // Pass 1: render the particles INTO the intermediate texture.
    const particlePass = Pass.fromShader("particles", particleShader);
    particlePass.addMesh(mesh);
    particlePass.addTarget(intermediate);
    particlePass.setClearColor([0.04, 0.05, 0.08, 1.0]);

    // Pass 2: sample the intermediate, apply postfx, render to the canvas.
    const postfxPass = Pass.fromShader("postfx", postfxShader);

    return {
        particleShader,
        postfxShader,
        particlePass,
        postfxPass,
        mesh,
        intermediate,
    };
}
// #endregion: setup

// #region: frame
export function frame(state, renderer, target, time, _size) {
    state.particleShader.set("time", time);
    state.postfxShader.set("time", time);
    renderer.render([state.particlePass, state.postfxPass], target);
}
// #endregion: frame
