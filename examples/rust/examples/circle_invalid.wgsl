struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0); // Validation should fail here
}

struct Circle {
    position: vec2<f32>,
    radius: f32,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> circle: Circle;

@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn main(pixel: VertexOutput) -> @location(0) vec4<f32> {
    let uv = pixel.coords.xy / resolution;
    let circle_pos = circle.position / resolution;
    let dist = distance(uv, circle_pos);
    let r = circle.radius / max(resolution.x, resolution.y);
    let circle_sdf = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
    return circle.color * circle_sdf;
}
