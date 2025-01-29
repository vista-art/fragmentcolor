struct Circle {
  position: vec2<f32>,
  radius: f32,
  color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> circle: Circle;

@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / resolution;
    let circle_pos = position / resolution;
    let dist = distance(uv, circle_pos);
    let r = radius / max(resolution.x, resolution.y);
    let circle = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
    return color * circle;
}
