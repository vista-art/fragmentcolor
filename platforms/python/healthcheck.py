import os
import platform
import importlib
import numpy as np
from fragmentcolor import (
    CompareFunction,
    Mipmap,
    Pass,
    Renderer,
    SamplerOptions,
    Shader,
    Size,
    TextureFormat,
)

# Debug diagnostics for CI
if os.environ.get("FC_HEALTHCHECK_VERBOSE") == "1":
    fc = importlib.import_module("fragmentcolor")
    try:
        print(f"fragmentcolor module path: {fc.__file__}")
    except Exception:
        pass
    print(f"platform: {platform.platform()}")


SET_VALUES_SHADER = """
struct Light { color: vec4<f32>, power: f32 };
struct Particles { items: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> count: u32;
@group(0) @binding(1) var<uniform> cell: vec2<i32>;
@group(0) @binding(2) var<uniform> rot: mat2x2<f32>;
@group(0) @binding(3) var<uniform> light: Light;
@group(0) @binding(4) var<uniform> offset: vec2<f32>;
@group(0) @binding(5) var<storage, read> particles: Particles;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> {
  let x = f32(count) + f32(cell.x) + rot[0].x + light.power + offset.x + particles.items[0].x;
  return vec4f(x, 0., 0., 1.);
}
"""


def check_shader_set_values():
    """Shader.set shapes plain Python values to each uniform's declared WGSL type."""
    shader = Shader(SET_VALUES_SHADER)

    def rejects(label, key, value):
        try:
            shader.set(key, value)
        except Exception as e:
            print(f"shader.set rejected {label}: {e}")
            return
        raise AssertionError(f"shader.set({key!r}, {label}) should raise")

    shader.set("count", 3)
    assert shader.get("count") == 3
    shader.set("cell", [2, -1])
    assert list(shader.get("cell")) == [2, -1]
    shader.set("rot", [1.0, 2.0, 3.0, 4.0])
    assert list(shader.get("rot")) == [1.0, 2.0, 3.0, 4.0]
    shader.set("offset", np.array([5, 6], dtype=np.int32))
    assert list(shader.get("offset")) == [5.0, 6.0]

    rejects("three numbers for a vec2", "offset", [1.0, 2.0, 3.0])
    assert list(shader.get("offset")) == [5.0, 6.0]
    rejects("3.5 for a u32", "count", 3.5)
    rejects("-1 for a u32", "count", -1)
    rejects("bytes for a vec2", "offset", bytes(8))

    blob = bytes(range(32))
    shader.set("particles", blob)
    assert shader.get("particles") == blob
    shader.set("particles", bytearray(32))
    assert shader.get("particles") == bytes(32)
    shader.set("particles", memoryview(blob))
    assert shader.get("particles") == blob

    shader.set("light.power", 0.5)
    light = shader.get("light")
    assert isinstance(light, bytes) and len(light) == 32
    assert np.frombuffer(light[16:20], dtype=np.float32)[0] == 0.5
    print("Shader.set shaped every value to its declared type")


def check_sampler_options(texture):
    """SamplerOptions builds from keywords, keeps the Rust defaults, and reads back."""
    defaults = SamplerOptions()
    assert (defaults.repeat_x, defaults.repeat_y, defaults.smooth) == (False, False, True)
    assert defaults.compare is None

    opts = SamplerOptions(repeat_x=True, compare=CompareFunction.LessEqual)
    assert opts.repeat_x and not opts.repeat_y and opts.smooth
    assert opts.compare == CompareFunction.LessEqual
    opts.smooth = False
    opts.compare = None
    assert not opts.smooth and opts.compare is None
    texture.set_sampler_options(opts)
    print("SamplerOptions and CompareFunction build from Python")


def check_size(renderer, texture):
    """Size reads its fields, and every Size parameter takes a Size back."""
    assert (texture.size.width, texture.size.height) == (2, 2)
    target = renderer.create_texture_target([16, 8])
    size = target.size
    assert isinstance(size, Size)
    assert (size.width, size.height, size.depth) == (16, 8, 1)
    size.width = 32
    target.resize(size)
    assert (target.size.width, target.size.height) == (32, 8)
    print("Size reads its fields and passes back in")


LOAD_TEXELS_SHADER = """
@group(0) @binding(0) var tex: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  return textureLoad(tex, vec2<i32>(pos.xy), 0);
}
"""


def check_ndarray_layouts(renderer):
    """Texture bytes read an ndarray in row-major order, whatever its memory layout."""
    image = np.arange(2 * 3 * 4, dtype=np.uint8).reshape(2, 3, 4) * 10
    layouts = {
        "Fortran-ordered": np.asfortranarray(image),
        "flipped": image[::-1].copy()[::-1],
        "sliced": np.repeat(image, 2, axis=1)[:, ::2],
    }
    for label, array in layouts.items():
        assert not array.flags["C_CONTIGUOUS"], label
        level = Mipmap.build(array, TextureFormat.Rgba, size=[3, 2]).level(0)
        assert level == image.tobytes(), f"Mipmap.build read a {label} array out of order"

    def rendered(array):
        shader = Shader(LOAD_TEXELS_SHADER)
        shader.set("tex", renderer.create_texture(array, format=TextureFormat.Rgba))
        target = renderer.create_texture_target([3, 2])
        renderer.render(shader, target)
        return target.get_image()

    expected = rendered(image)
    for label, array in layouts.items():
        assert np.array_equal(rendered(array), expected), f"create_texture read a {label} array out of order"
    print("ndarray texture bytes read in row-major order for every layout")


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

    # Render with a Pass, then render a sequence of passes (any iterable of Pass is renderable).
    rpass = Pass("single pass")
    rpass.add_shader(shader)
    renderer.render(rpass, target)

    shader.set("circle.radius", 30.0)
    renderer.render(rpass, target)

    rpass2 = Pass("second pass")
    rpass2.add_shader(shader)
    renderer.render([rpass, rpass2], target)

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
    check_sampler_options(tex)
    check_size(renderer, tex)
    check_ndarray_layouts(renderer)

    # Push constants smoke: solid color via var<immediate>
    pc_shader = Shader("""
struct PC { color: vec4<f32> };
var<immediate> pc: PC;
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

    check_shader_set_values()


if __name__ == "__main__":
    run()

    # Auto-generated: run all extracted & translated examples from docs/api
    from platforms.python.examples.main import run_all
    run_all()
