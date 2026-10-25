@file:Suppress("UNUSED_PARAMETER")

// Step 4: the program owns the triangle's corners, each with its own color.
// Call `setup` once, then call `frame` on every display refresh, such as from a `Choreographer` callback.

import org.fragmentcolor.*

const val GRADIENT = """
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
"""

// #region: setup
fun setup(): Shader {
    val shader = Shader(GRADIENT)

    val mesh = Mesh()
    mesh.addVertices(listOf(
        Vertex(listOf(-0.7f, -0.404f, 0.0f)).set("color", listOf(0.95f, 0.30f, 0.42f)),
        Vertex(listOf(0.7f, -0.404f, 0.0f)).set("color", listOf(0.30f, 0.85f, 0.55f)),
        Vertex(listOf(0.0f, 0.808f, 0.0f)).set("color", listOf(0.30f, 0.55f, 0.95f)),
    ))
    shader.addMesh(mesh)
    return shader
}
// #endregion: setup

// #region: frame
fun frame(shader: Shader, renderer: Renderer, target: WindowTarget, time: Float) {
    val size = target.size()
    shader.set("resolution", listOf(size.width.toFloat(), size.height.toFloat()))
    renderer.render(shader, target)
}
// #endregion: frame
