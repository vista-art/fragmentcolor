// FragmentColor — Hello, triangle.
//
// A simple self-contained triangle:
// three positions baked into the vertex shader, aspect-corrected via
// the `resolution` uniform so the shape stays a triangle on any canvas
// size, plus one color read from a uniform provided by the user.

struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.5),
        vec2<f32>( 0.6, -0.5),
        vec2<f32>( 0.0,  0.7),
    );
    // Aspect-correct so the triangle never stretches with the canvas.
    // `max(..., 1.0)` keeps things sane if `resolution` hasn't been set
    // yet (uniforms default to zero, and we'd otherwise divide by zero).
    let res = max(resolution, vec2<f32>(1.0));
    let aspect = res.x / res.y;
    var pos = p[i];
    if (aspect > 1.0) {
        pos.x = pos.x / aspect;
    } else {
        pos.y = pos.y * aspect;
    }
    var out: VOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
