def example_shader():
    from fragmentcolor import Shader
    return Shader("
struct VertexOutput {\n    @builtin(position) coords: vec4<f32>,\n}\n@vertex\nfn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {\n    const vertices = array( vec2(-1.,-1.), vec2(3.,-1.), vec2(-1.,3.) );\n    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));\n}\n@fragment\nfn main() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}\n" )
