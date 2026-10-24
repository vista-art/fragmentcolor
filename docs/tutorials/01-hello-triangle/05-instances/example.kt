// Step 5: the GPU draws one mesh 15000 times, each copy with its own place, phase and tint.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import org.fragmentcolor.*
import kotlin.math.PI
import kotlin.random.Random

const val COPIES = 15000

const val PARTICLES = """
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
// #endregion: shader
"""

// #region: setup
fun setup(): Shader {
    val shader = Shader(listOf("easing/in_out_sine", PARTICLES))

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
