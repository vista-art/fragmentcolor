struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 1.0);
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

struct Screen {
    resolution: vec2<f32>,
    antialiaser: f32,
    _padding: f32,
};
@group(0) @binding(0)
var<uniform> screen: Screen;

struct Circle {
    position: vec2<f32>,
    radius: f32,
    border: f32,
    color: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> circle: Circle;

@fragment 
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    let aa = screen.antialiaser;
    let radius = circle.radius;
    let border = circle.border;

    let normalized = in.position.xy / screen.resolution;
    var uv = (normalized * 2.0) - vec2(1.0);
    uv.x *= screen.resolution.x / screen.resolution.y;

    let dist = distance(uv, circle.position.xy);
    let alpha = (1.0 - smoothstep(border - aa, border + aa, abs(dist - radius))) * circle.color.a;

    out.color = vec4<f32>(circle.color.rgb, alpha);

    return out;
}
