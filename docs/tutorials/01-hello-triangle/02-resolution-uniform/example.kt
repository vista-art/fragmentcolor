// Step 2: a second uniform, `resolution`, keeps the triangle's shape on any screen.
// Call setup once, then frame on every display refresh, such as from a Choreographer callback.

import org.fragmentcolor.*
import kotlin.math.cos
import kotlin.math.sin

const val TRIANGLE = """
// #region: shader
struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.4),
        vec2<f32>( 0.7, -0.4),
        vec2<f32>( 0.0,  0.8),
    );
    // Shrink the longer axis so one unit spans the same pixels on both.
    let aspect = resolution.x / resolution.y;
    var pos = corners[i];
    if (aspect > 1.0) { pos.x = pos.x / aspect; }
    else              { pos.y = pos.y * aspect; }

    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
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

    val size = target.size()
    shader.set("resolution", listOf(size.width.toFloat(), size.height.toFloat()))
    renderer.render(shader, target)
}
// #endregion: frame
