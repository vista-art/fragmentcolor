"""Step 6: the particles render into a texture, and a second pass grades it for the screen.

From the repository root, run `python docs/tutorials/01-hello-triangle/host.py 06-postfx-pass`.
"""

from dataclasses import dataclass
from math import tau
from random import uniform

from fragmentcolor import Instance, Mesh, Pass, Shader, Vertex

COPIES = 15000

PARTICLES = """
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
"""

POSTFX = """
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
"""


@dataclass
class State:
    """What `frame` needs from `setup`."""

    particles: Shader
    postfx: Shader
    output: Pass


# #region: setup
def setup(renderer):
    particles = Shader(["easing/in_out_sine", PARTICLES])

    mesh = Mesh()
    mesh.add_vertices([
        Vertex([-0.7, -0.4, 0.0]).set("color", [0.95, 0.30, 0.42]),
        Vertex([0.7, -0.4, 0.0]).set("color", [0.30, 0.85, 0.55]),
        Vertex([0.0, 0.8, 0.0]).set("color", [0.30, 0.55, 0.95]),
    ])
    for _ in range(COPIES):
        center = [uniform(-1.0, 1.0), uniform(-1.0, 1.0)]
        phase = uniform(0.0, tau)
        tint = [uniform(0.2, 1.0), uniform(0.2, 1.0), uniform(0.2, 1.0)]
        mesh.add_instance(
            Instance()
            .set("center", center)
            .set("phase", phase)
            .set("tint", tint)
        )
    particles.add_mesh(mesh)

    # The particles draw into this texture, and the postfx shader reads it.
    scene = renderer.create_texture_target([1024, 1024])

    postfx = Shader([
        "postfx/chromatic_offsets",
        "color/tonemap_filmic",
        "postfx/vignette",
        "postfx/film_grain",
        POSTFX,
    ])
    postfx.set("scene", scene.texture())

    particle_pass = Pass.from_shader("particles", particles)
    particle_pass.set_target(scene)
    particle_pass.set_clear_color([0.04, 0.05, 0.08, 1.0])

    output = Pass.from_shader("postfx", postfx)
    output.require(particle_pass)

    return State(particles, postfx, output)
# #endregion: setup


# #region: frame
def frame(state, renderer, target, time):
    size = target.size()
    state.particles.set("resolution", [size.width, size.height])
    state.particles.set("time", time)
    state.postfx.set("time", time)
    renderer.render(state.output, target)
# #endregion: frame
