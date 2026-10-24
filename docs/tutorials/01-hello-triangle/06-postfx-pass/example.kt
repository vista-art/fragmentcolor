// Step 6: the particles render into a texture, and a second pass grades it for the screen.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import org.fragmentcolor.*
import kotlin.math.PI
import kotlin.random.Random

const val COPIES = 15000

const val PARTICLES = """
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

const val POSTFX = """
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
    // One triangle large enough to cover the whole target.
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
    // Sample each channel at its own offset.
    let offsets = chromatic_offsets(in.uv, 0.006);
    let red = textureSample(scene, scene_sampler, offsets[0]).r;
    let green = textureSample(scene, scene_sampler, offsets[1]).g;
    let blue = textureSample(scene, scene_sampler, offsets[2]).b;

    // Halve the exposure, then compress the highlights.
    let graded = tonemap_filmic(vec3<f32>(red, green, blue) * 0.5);
    let edges = vignette(in.uv, 0.55, 0.40);
    let grain = film_grain(in.uv, time) * 0.05;
    return vec4<f32>(graded * edges + grain, 1.0);
}
// #endregion: postfx-shader
"""

/** What `frame` needs from `setup`. */
class State(val particles: Shader, val postfx: Shader, val output: Pass)

// #region: setup
suspend fun setup(renderer: Renderer): State {
    val particles = Shader(listOf("easing/in_out_sine", PARTICLES))

    val mesh = Mesh()
    mesh.addVertices(listOf(
        Vertex(listOf(-0.7f, -0.404f, 0.0f)).set("color", listOf(0.95f, 0.30f, 0.42f)),
        Vertex(listOf(0.7f, -0.404f, 0.0f)).set("color", listOf(0.30f, 0.85f, 0.55f)),
        Vertex(listOf(0.0f, 0.808f, 0.0f)).set("color", listOf(0.30f, 0.55f, 0.95f)),
    ))
    repeat(COPIES) {
        val center = listOf(Random.nextFloat() * 2f - 1f, Random.nextFloat() * 2f - 1f)
        val phase = Random.nextFloat() * 2f * PI.toFloat()
        val tint = List(3) { 0.2f + Random.nextFloat() * 0.8f }
        mesh.addInstance(
            Instance()
                .set("center", center)
                .set("phase", phase)
                .set("tint", tint)
        )
    }
    particles.addMesh(mesh)

    // The particle pass draws into this texture for `postfx` to read.
    val scene = renderer.createTextureTarget(listOf(1024u, 1024u))

    val postfx = Shader(listOf(
        "postfx/chromatic_offsets",
        "color/tonemap_filmic",
        "postfx/vignette",
        "postfx/film_grain",
        POSTFX,
    ))
    postfx.set("scene", scene.texture())

    val particlePass = Pass.fromShader("particles", particles)
    particlePass.setTarget(scene)
    particlePass.setClearColor(listOf(0.04f, 0.05f, 0.08f, 1.0f))

    val output = Pass.fromShader("postfx", postfx)
    output.require(particlePass)

    return State(particles, postfx, output)
}
// #endregion: setup

// #region: frame
fun frame(state: State, renderer: Renderer, target: WindowTarget, time: Float) {
    val size = target.size()
    state.particles.set("resolution", listOf(size.width.toFloat(), size.height.toFloat()))
    state.particles.set("time", time)
    state.postfx.set("time", time)
    renderer.render(state.output, target)
}
// #endregion: frame
