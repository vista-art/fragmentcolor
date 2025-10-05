import os
import platform
import importlib
import numpy as np
from fragmentcolor import Renderer, Shader, Pass, Frame

# Debug diagnostics for CI
if os.environ.get("FC_HEALTHCHECK_VERBOSE") == "1":
    fc = importlib.import_module("fragmentcolor")
    try:
        print(f"fragmentcolor module path: {fc.__file__}")
    except Exception:
        pass
    print(f"platform: {platform.platform()}")


def run():
    """Simple smoke tests before running full examples"""

    # Tag this process for GPU error context
    os.environ["FC_RUNNER"] = "python"
    os.environ["FC_CURRENT_TEST"] = "platforms.python.healthcheck"

    renderer = Renderer()
    target = renderer.create_texture_target((32, 64))
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

    shader.set("resolution", [32.0, 64.0])
    shader.set("circle.radius", 10.0)
    shader.set("circle.color", [1.0, 0.0, 0.0, 0.8])
    shader.set("circle.border", 2.0)
    shader.set("circle.position", [0.0, 0.0])

    # Multiple render calls succeed
    renderer.render(shader, target)
    renderer.render(shader, target)
    renderer.render(shader, target)

    # Images are ndarrays of uint8 with correct shape
    image = target.get_image()
    print(f"Image shape: {image.shape}, dtype: {image.dtype}")
    assert image.ndim == 3
    assert image.shape[0] == 32
    assert image.shape[1] == 64
    assert image.shape[2] == 4
    assert image.dtype == np.dtype(np.uint8)

    # Update uniform and render again
    shader.set("circle.radius", 20.0)
    renderer.render(shader, target)

    # Render with a Pass and a Frame
    rpass = Pass("single pass")
    rpass.add_shader(shader)
    renderer.render(rpass, target)

    shader.set("circle.radius", 30.0)
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

    # Test texture creation and shader.set parity
    tex_shader = Shader("""
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    let p = array(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
    let uv = array(vec2f(0.,1.), vec2f(2.,1.), vec2f(0.,-1.));
    var out: VOut;
    out.pos = vec4f(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
""")

    # Test numpy ndarray path
    tex_arr = np.array([
        [[255, 0, 0, 255], [0, 255, 0, 255]],
        [[0, 0, 255, 255], [255, 255, 255, 255]],
    ], dtype=np.uint8)
    tex = renderer.create_texture(tex_arr)
    print(f"Created texture from numpy array: shape={tex_arr.shape}")

    tex_shader.set("tex", tex)
    tex_shader.set("resolution", [32.0, 32.0])
    print("Set texture on shader successfully")

    tex_target = renderer.create_texture_target((32, 32))
    renderer.render(tex_shader, tex_target)
    tex_img = tex_target.get_image()
    print(f"Rendered textured shader: shape={tex_img.shape}")

    # SamplerOptions via dict (conversion path)
    tex.set_sampler_options(
        {"repeat_x": True, "repeat_y": False, "smooth": True, "compare": None})

    # Push constants smoke: solid color via var<push_constant>
    pc_shader = Shader("""
struct PC { color: vec4<f32> };
var<push_constant> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return pc.color; }
""")
    pc_shader.set("pc.color", [0.0, 0.0, 1.0, 1.0])
    pc_target = renderer.create_texture_target((8, 8))
    renderer.render(pc_shader, pc_target)
    pc_img = pc_target.get_image()
    print(f"Push constant render: first pixel={pc_img[0, 0, :]}")
    assert pc_img.shape[0] >= 1 and pc_img.shape[1] >= 1 and pc_img.shape[2] == 4
    assert int(pc_img[0, 0, 0]) == 0 and int(pc_img[0, 0, 1]) == 0 and int(
        pc_img[0, 0, 2]) == 255 and int(pc_img[0, 0, 3]) == 255

    print("Headless Python render completed successfully")


if __name__ == "__main__":
    run()

    # Auto-generated: run all extracted & translated examples from docs/api
    from platforms.python.examples.main import run_all
    run_all()
