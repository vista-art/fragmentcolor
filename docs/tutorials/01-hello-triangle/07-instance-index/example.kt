// Step 7: a million triangles, each placed by the GPU from nothing but its index.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import org.fragmentcolor.*

const val COPIES = 1_000_000u

const val GALAXY = """
// #region: shader
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@group(0) @binding(1) var<uniform> time: f32;

// PCG hash: mixes the bits of a number until the result looks random.
fn hash(value: u32) -> u32 {
    let state = value * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// The same seed always gives the same number between 0 and 1.
fn random(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0;
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @builtin(instance_index) index: u32,
) -> VOut {
    // Four random numbers per copy, drawn from its index.
    let radius = random(index * 4u);
    let arm = f32(hash(index * 4u + 1u) % 3u) * 2.094;
    let spread = (random(index * 4u + 2u) - 0.5) * (1.2 - radius);
    let brightness = 0.4 + 0.6 * random(index * 4u + 3u);

    // Three arms that curl outward, turning slowly as one disc.
    let angle = arm + radius * 5.0 + spread - time * 0.15;
    let center = vec2<f32>(cos(angle), sin(angle)) * radius * 0.95;

    // Shrink the longer axis so the disc stays round.
    let fit = min(resolution.x, resolution.y) / resolution;
    // Measured in pixels, each triangle is about four wide.
    let shape = position * 6.0 / resolution;

    var out: VOut;
    out.pos = vec4<f32>(center * fit + shape, 0.0, 1.0);
    let core = vec3<f32>(1.0, 0.8, 0.55);
    let rim = vec3<f32>(0.35, 0.55, 1.0);
    out.color = mix(core, rim, radius) * brightness;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
// #endregion: shader
"""

// #region: setup
fun setup(): Shader {
    val shader = Shader(GALAXY)

    val mesh = Mesh()
    mesh.addVertices(listOf(listOf(-0.7f, -0.404f), listOf(0.7f, -0.404f), listOf(0.0f, 0.808f)))
    mesh.setInstanceCount(COPIES)

    shader.addMesh(mesh)
    return shader
}
// #endregion: setup

// #region: frame
fun frame(shader: Shader, renderer: Renderer, target: WindowTarget, time: Float) {
    val size = target.size()
    shader.set("resolution", listOf(size.width.toFloat(), size.height.toFloat()))
    shader.set("time", time)
    renderer.render(shader, target)
}
// #endregion: frame
