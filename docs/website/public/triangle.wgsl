// FragmentColor — Hello, triangle.
//
// A simple self-contained triangle:
// three positions baked into the vertex shader,
// one color read from a uniform provided by the user.

struct VOut { @builtin(position) pos: vec4<f32> };

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    // Equilateral triangle in NDC. apex_y / base_y come from
    // height = side * sqrt(3) / 2 with side = 1.2.
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.35),
        vec2<f32>( 0.6, -0.35),
        vec2<f32>( 0.0,  0.69),
    );
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
