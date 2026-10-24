// Step 1: a shader with one uniform, drawn every frame in a new color.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import org.fragmentcolor.*
import kotlin.math.cos
import kotlin.math.sin

const val TRIANGLE = """
// #region: shader
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Three corners of an equilateral triangle, centered on the origin.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.404),
        vec2<f32>( 0.7, -0.404),
        vec2<f32>( 0.0,  0.808),
    );
    var out: VOut;
    out.pos = vec4<f32>(corners[i], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
// #endregion: shader
"""

// #region: setup
fun setup(): Shader {
    val shader = Shader(TRIANGLE)
    shader.set("color", listOf(0.95f, 0.30f, 0.42f, 1.0f))
    return shader
}
// #endregion: setup

// #region: frame
fun frame(shader: Shader, renderer: Renderer, target: WindowTarget, time: Float) {
    val red = 0.5f + 0.45f * sin(time * 0.7f)
    val green = 0.5f + 0.45f * cos(time * 0.5f + 1.7f)
    val blue = 0.5f + 0.45f * sin(time * 0.9f + 3.1f)
    shader.set("color", listOf(red, green, blue, 1.0f))
    renderer.render(shader, target)
}
// #endregion: frame
