import os
import platform
import importlib
from fragmentcolor import Renderer, Shader, Pass, Frame

# Optional debug diagnostics for CI
if os.environ.get("FC_HEALTHCHECK_VERBOSE") == "1":
    fc = importlib.import_module("fragmentcolor")
    try:
        print(f"fragmentcolor module path: {fc.__file__}")
    except Exception:
        pass
    print(f"platform: {platform.platform()}")


def run():
    renderer = Renderer()
    target = renderer.create_texture_target((64, 64))
    shader = Shader("""
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array(
        vec2( -1., -1.),
        vec2(  3., -1.),
        vec2( -1.,  3.)
    );
    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}

struct Circle {
    position: vec2<f32>,
    radius: f32,
    border: f32,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> circle: Circle;

@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

    @fragment
fn main(pixel: VertexOutput) -> @location(0) vec4<f32> {
    let normalized_coords = pixel.coords.xy / resolution;
    var uv = -1.0 + 2.0 * normalized_coords;
    if (resolution.x > resolution.y) {
        uv.x *= resolution.x / resolution.y;
    } else {
        uv.y *= resolution.y / resolution.x;
    }
    let circle_pos = circle.position / resolution;
    let dist = distance(uv, circle_pos);
    let r = circle.radius / min(resolution.x, resolution.y);
    let aa = 2. / min(resolution.x, resolution.y);
    let border = circle.border / min(resolution.x, resolution.y);

    if (dist > r + (border + aa)) { discard; }

    let circle_sdf = 1.0 - smoothstep(border - aa, border + aa, abs(dist - r));
    let a = circle.color.a * circle_sdf;
    return vec4<f32>(circle.color.rgb * a, a);
}
""")

    shader.set("resolution", [64.0, 64.0])
    shader.set("circle.radius", 10.0)
    shader.set("circle.color", [1.0, 0.0, 0.0, 0.8])
    shader.set("circle.border", 2.0)
    shader.set("circle.position", [0.0, 0.0])

    renderer.render(shader, target)

    rpass = Pass("single pass")
    rpass.add_shader(shader)
    renderer.render(rpass, target)

    frame = Frame()
    frame.add_pass(rpass)
    renderer.render(frame, target)

    # Additional API coverage for docs
    radius = shader.get("circle.radius")
    uniforms = shader.list_uniforms()
    keys = shader.list_keys()

    print(f"Shader.get('circle.radius'): {radius}")
    print(f"Shader.list_uniforms: {uniforms}")
    print(f"Shader.list_keys: {keys}")
    print("Headless Python render completed successfully")


if __name__ == "__main__":
    run()

    # Auto-generated: run all extracted examples
    from platforms.python.examples.main import run_all as __run_all
    __run_all()
