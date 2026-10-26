import os
import platform
import importlib
from pathlib import Path
import numpy as np
from fragmentcolor import (
    Camera,
    CompareFunction,
    DeviceError,
    ErrorKind,
    FragmentColorError,
    Instance,
    InvalidInputError,
    Light,
    MalformedImageError,
    Material,
    MaterialError,
    Mesh,
    OccludedSurfaceError,
    Mipmap,
    Model,
    Pass,
    RenderCanvasTarget,
    Renderer,
    SamplerOptions,
    Scene,
    SceneLoadError,
    ScreenRegion,
    Shader,
    Size,
    TextureFormat,
    TextureSlot,
    TextureUsage,
    Quad,
    Vertex,
    WindowTarget,
)
from platforms.python.examples.main import run_all, take_gpu_errors

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
    assert (texture.size().width, texture.size().height) == (2, 2)
    target = renderer.create_texture_target([16, 8])
    size = target.size()
    assert isinstance(size, Size)
    assert (size.width, size.height, size.depth) == (16, 8, 1)
    size.width = 32
    target.resize(size)
    assert (target.size().width, target.size().height) == (32, 8)
    # A size's fields come in any mapping, as the stub's Mapping hint says.
    import collections
    import types
    for fields in ({"width": 8, "height": 4}, collections.UserDict(width=8, height=4),
                   types.MappingProxyType({"width": 8, "height": 4, "depth": 1})):
        target.resize(fields)
        assert (target.size().width, target.size().height) == (8, 4), f"{type(fields).__name__} size"
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


EDGES_SHADER = """
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4f(select(0., 1., pos.x < 1.), select(0., 1., pos.y < 1.), 0., 1.);
}
"""


def check_image_layout(renderer):
    """get_image returns (height, width, 4) rows top to bottom, the layout create_texture reads."""
    target = renderer.create_texture_target([3, 2])
    renderer.render(Shader(EDGES_SHADER), target)
    image = target.get_image()
    assert image.shape == (2, 3, 4), f"get_image shape {image.shape}, expected (height, width, 4)"
    assert image.flags["C_CONTIGUOUS"]
    assert (image[:, 0, 0] == 255).all() and (image[:, 1:, 0] == 0).all(), "red is not the left column"
    assert (image[0, :, 1] == 255).all() and (image[1:, :, 1] == 0).all(), "green is not the top row"
    texels = target.texture().get_image()
    assert texels.shape == (2, 3, 4), f"Texture.get_image shape {texels.shape}, expected (height, width, 4)"
    assert np.array_equal(texels, image), "Texture.get_image differs from TextureTarget.get_image"

    texture = renderer.create_texture(image, format=TextureFormat.Rgba)
    assert (texture.size().width, texture.size().height) == (3, 2)
    shader = Shader(LOAD_TEXELS_SHADER)
    shader.set("tex", texture)
    copy = renderer.create_texture_target([3, 2])
    renderer.render(shader, copy)
    assert np.array_equal(copy.get_image(), image), "create_texture(get_image()) changed the image"
    print("get_image returns (height, width, 4) and round-trips through create_texture")


def check_tight_writes(renderer):
    """Texture writes take tightly packed rows at any width."""
    # 2 pixels * 4 bytes = 8 bytes per row, far from the 256 a buffer copy needs
    texture = renderer.create_texture([255] * 16, size=[2, 2])
    texture.write(bytes(16))
    texture.write(np.zeros((2, 2, 4), dtype=np.uint8))

    storage = renderer.create_storage_texture([3, 2], TextureFormat.Rgba)
    pixels = bytes(range(3 * 2 * 4))
    storage.write(pixels)
    assert bytes(storage.get_image()) == pixels, "write read rows at the wrong stride"

    # Pixels (1, 1) and (2, 1) of the 3x2 texture
    patch = bytes(range(100, 108))
    storage.write_region(patch, [1, 1, 2, 1])
    image = bytes(storage.get_image())
    assert image[16:24] == patch, "write_region read rows at the wrong stride"
    assert image[:16] == pixels[:16], "write_region wrote outside its region"
    print("Texture.write and write_region take tightly packed rows")


def check_error_types(renderer):
    """Renderer and texture errors raise FragmentColorError, as shader errors do."""
    def raises(label, call):
        try:
            call()
        except FragmentColorError as e:
            print(f"{label} raised FragmentColorError: {e}")
            return
        raise AssertionError(f"{label} should raise FragmentColorError")

    texture = renderer.create_texture([255] * 16, size=[2, 2])
    raises("Texture.write with 4 of 16 bytes", lambda: texture.write(bytes(4)))
    renderer.unregister_texture(texture.id())
    raises("a second unregister_texture", lambda: renderer.unregister_texture(texture.id()))
    # A hidden window can't be made headless, so only the class is checked.
    assert issubclass(OccludedSurfaceError, FragmentColorError)

    # A pipeline the GPU refuses raises DeviceError, with its kind and the step's label.
    # The fragment stage reads @location(0), which the vertex stage never writes.
    unfed = Shader("""
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
  return VOut(vec4f(p[i], 0., 1.));
}
@fragment fn fs_main(@location(0) tint: vec4<f32>) -> @location(0) vec4<f32> { return tint; }
""")
    target = renderer.create_texture_target([8, 8])
    try:
        renderer.render(unfed, target)
    except DeviceError as e:
        assert isinstance(e, FragmentColorError)
        assert e.kind == ErrorKind.Validation, f"kind {e.kind}"
        assert e.label == "render pipeline", f"label {e.label}"
        assert str(e).startswith("Device error (validation) in 'render pipeline'"), str(e)
        assert e.detail and str(e).endswith(f": {e.detail}"), f"detail {e.detail}"
        print(f"render of a refused pipeline raised DeviceError: {e}")
    else:
        raise AssertionError("render of a refused pipeline should raise DeviceError")


SOLID_SHADER = """
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(1., 0., 0., 1.); }
"""

def check_require_graph(renderer):
    """Rendering a Pass runs the passes it requires, as in every other binding."""
    side = renderer.create_texture_target([4, 4])
    color = Pass("color")
    color.add_shader(Shader(SOLID_SHADER))
    color.set_target(side)
    compose = Pass("compose")
    compose.add_shader(Shader(SOLID_SHADER))
    compose.require(color)

    renderer.render(compose, renderer.create_texture_target([4, 4]))
    pixel = side.get_image()[0, 0]
    assert list(pixel) == [255, 0, 0, 255], f"the required pass did not run, got {pixel}"

    fan_in = Pass("fan_in")
    fan_in.require([color, compose])
    for bad in ("color", [color, 3]):
        try:
            fan_in.require(bad)
        except TypeError:
            continue
        raise AssertionError(f"require({bad!r}) should raise TypeError")
    print("Pass.require runs the required passes")


SAMPLE_SHADER = """
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var soft: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  return textureSample(source, soft, pos.xy / 8.0);
}
"""

def check_pass_targets(renderer):
    """A target with a mip chain raises InvalidInputError, and render refuses a pass
    that samples its own target, or a target from another Renderer, before any GPU work."""
    chain = renderer.create_texture(bytes(64), size=[4, 4], usage=TextureUsage.RENDER_ATTACHMENT)
    raises_invalid("Pass.set_target with a mip chain", lambda: Pass("chain").set_target(chain))

    def refuses(label, words, call):
        try:
            call()
        except DeviceError as e:
            raise AssertionError(f"{label} reached the GPU: {e}")
        except FragmentColorError as e:
            assert words in str(e), str(e)
            print(f"{label} raised FragmentColorError: {e}")
            return
        raise AssertionError(f"{label} should raise FragmentColorError")

    target = renderer.create_texture_target([8, 8])
    feedback = Shader(SAMPLE_SHADER)
    feedback.set("source", target.texture())
    refuses("sampling the render target", "Texture conflict", lambda: renderer.render(feedback, target))

    foreign = Renderer().create_texture_target([8, 8])
    refuses("a target from another Renderer", "another Renderer",
            lambda: renderer.render(Shader(SOLID_SHADER), foreign))
    refuses("a Renderer with no context", "another Renderer",
            lambda: Renderer().render(Shader(SOLID_SHADER), target))

    # A pass error keeps its class whichever call raises it: render raises
    # InvalidInputError for a target format the shader can't fill, as set_target does.
    usage = TextureUsage.RENDER_ATTACHMENT | TextureUsage.TEXTURE_BINDING
    integer = renderer.create_storage_texture([8, 8], TextureFormat.Rgba32Uint, usage=usage)
    floats = Pass("floats into integers")
    floats.add_shader(Shader(SOLID_SHADER))
    floats.set_target(integer)
    raises_invalid("render of vec4<f32> into Rgba32Uint", lambda: renderer.render(floats, target))

    wide = Pass("wide viewport")
    wide.add_shader(Shader(SOLID_SHADER))
    wide.set_viewport([0, 0, 1 << 24, 8])
    raises_invalid("render of a viewport past the device limit", lambda: renderer.render(wide, target))
    renderer.render(Shader(SOLID_SHADER), target)


def check_value_types():
    """Value types compare by value, and their fields read back."""
    chain = Mipmap.build(bytes(4 * 4 * 4), TextureFormat.Rgba8UnormSrgb, [4, 4])
    assert chain.format() == TextureFormat.Rgba8UnormSrgb
    assert len(chain.levels()) == chain.count() == 3
    assert chain.levels()[0] == chain.level(0)
    assert Size(4, 2) == Size(4, 2) and Size(4, 2) != Size(2, 4)
    region = ScreenRegion((1, 2), (32, 16))
    assert region == ScreenRegion.from_region(1, 2, 32, 16)
    assert (region.min_x, region.max_y, region.width(), region.height()) == (1, 18, 32, 16)
    assert ScreenRegion.from_tuple((64, 32)).width() == 64
    assert SamplerOptions(smooth=False) == SamplerOptions(smooth=False)
    assert RenderCanvasTarget is WindowTarget
    print("Value types compare by value")


def check_number_errors(renderer):
    """Counts, indices, sizes and regions take ints and raise InvalidInputError for one they can't hold."""
    def raises(kind, label, call):
        try:
            call()
        except kind as e:
            print(f"{label} raised {kind.__name__}: {e}")
            return
        raise AssertionError(f"{label} should raise {kind.__name__}")

    rpass, mesh, scene = Pass("numbers"), Mesh(), Scene()
    scene.add_pass(rpass)
    chain = Mipmap.build(bytes(4 * 4 * 4), TextureFormat.Rgba8Unorm, [4, 4])
    texture = renderer.create_texture(bytes(4 * 4 * 4), size=[4, 4])
    target = renderer.create_texture_target([4, 4])
    model = Model(Mesh(), Material.pbr())
    material = Material.pbr()
    calls = {
        "Pass.set_compute_dispatch": lambda n: rpass.set_compute_dispatch(n, 1, 1),
        "Pass.set_viewport": lambda n: rpass.set_viewport([0, 0, n, 4]),
        "Mesh.set_instance_count": lambda n: mesh.set_instance_count(n),
        "Mesh.set_indices": lambda n: mesh.set_indices([0, n]),
        "Scene.get_pass": lambda n: scene.get_pass(n),
        "Scene.add_to": lambda n: scene.add_to(n, model),
        "Mipmap.level": lambda n: chain.level(n),
        "Texture.write_region": lambda n: texture.write_region(bytes(4), [n, 0, 1, 1]),
        "Texture.write_region dict": lambda n: texture.write_region(bytes(4), {"min_x": n, "max_x": 1, "max_y": 1}),
        "Size": lambda n: Size(n, 2),
        "Size.depth": lambda n: setattr(Size(4, 2), "depth", n),
        "TextureTarget.resize": lambda n: target.resize([n, 4]),
        "create_texture_target": lambda n: renderer.create_texture_target({"width": n, "height": 4}),
        "ScreenRegion": lambda n: ScreenRegion((0, 0), (n, 4)),
        "ScreenRegion.from_region": lambda n: ScreenRegion.from_region(0, 0, n, 4),
        "ScreenRegion.from_tuple": lambda n: ScreenRegion.from_tuple((n, 4)),
        "Material.texture_uv": lambda n: material.texture_uv(TextureSlot.Occlusion, n),
    }
    # An int of any size outside 0 to 2**32 - 1 is a value error, never OverflowError.
    for name, call in calls.items():
        for bad in (-1, 2**32, 2**128, 10**400):
            raises(InvalidInputError, f"{name}({bad})", lambda: call(bad))
        # A float or a bool is the wrong type, even a whole one.
        raises(TypeError, f"{name}(1.5)", lambda: call(1.5))
        raises(TypeError, f"{name}(2.0)", lambda: call(2.0))
        raises(TypeError, f"{name}(True)", lambda: call(True))
    assert issubclass(InvalidInputError, FragmentColorError)

    # A texture id reads as a u64, so past that range it is a value error too.
    for bad in (-1, 2**64, 10**400):
        raises(InvalidInputError, f"Renderer.read_texture({bad})", lambda: renderer.read_texture(bad))
        raises(InvalidInputError, f"Renderer.unregister_texture({bad})", lambda: renderer.unregister_texture(bad))
    raises(TypeError, "Renderer.read_texture(1.5)", lambda: renderer.read_texture(1.5))
    raises(TypeError, "Renderer.read_texture(True)", lambda: renderer.read_texture(True))
    readable = renderer.create_texture(bytes(64), size=[4, 4], usage=TextureUsage.COPY_SRC)
    assert len(renderer.read_texture(readable.id().id)) == len(renderer.read_texture(readable.id())) == 64
    raises(InvalidInputError, "Pass.set_viewport([0, 0, 4])", lambda: rpass.set_viewport([0, 0, 4]))

    # A UV set past 1 fits a u32, but no texture can sample it.
    assert issubclass(MaterialError, FragmentColorError) and issubclass(MaterialError, ValueError)
    raises(MaterialError, "Material.texture_uv(2)", lambda: material.texture_uv(TextureSlot.Occlusion, 2))
    assert isinstance(material.texture_uv(TextureSlot.Occlusion, np.uint32(1)), Material)

    # An offset or scale takes two numbers. Another count is a value error, and a string the wrong type.
    transforms = {
        "Material.texture_transform offset": lambda v: material.texture_transform(TextureSlot.Normal, v, [1.0, 1.0], 0.0),
        "Material.texture_transform scale": lambda v: material.texture_transform(TextureSlot.Normal, [0.0, 0.0], v, 0.0),
        "Material.uv_transform offset": lambda v: material.uv_transform(v, [1.0, 1.0], 0.0),
        "Material.uv_transform scale": lambda v: material.uv_transform([0.0, 0.0], v, 0.0),
    }
    for name, apply in transforms.items():
        raises(InvalidInputError, f"{name} with 3 numbers", lambda: apply([0.0, 0.0, 0.0]))
        raises(TypeError, f"{name} with a string", lambda: apply("ab"))
        for bad in (float("nan"), float("inf"), 1e39):
            raises(InvalidInputError, f"{name} with {bad}", lambda: apply([bad, 1.0]))
        assert isinstance(apply(np.array([0.5, 2.0])), Material)

    size = Size(4, 2)
    for field in ("width", "height", "depth"):
        raises(InvalidInputError, f"Size.{field} = -1", lambda: setattr(size, field, -1))
    assert (size.width, size.height, size.depth) == (4, 2, None), "a refused write changed the Size"
    size.depth = 3
    size.depth = None
    assert size.depth is None

    # NumPy integers count as ints.
    rpass.set_viewport([0, 0, np.uint32(4), 4])
    rpass.set_compute_dispatch(np.int64(64), 1, 1)
    mesh.set_indices(np.array([0, 1, 2], dtype=np.uint32))
    assert Size(np.uint32(4), 2).width == 4
    print("Counts, indices, sizes and regions take ints and refuse ones they can't hold")


def check_empty_regions():
    """A region with a side of 0, or a far edge past 2**32 - 1, raises InvalidInputError."""
    rpass = Pass("empty")
    for label, call in (
        ("ScreenRegion((0, 0), (0, 4))", lambda: ScreenRegion((0, 0), (0, 4))),
        ("ScreenRegion.from_region(1, 1, 4, 0)", lambda: ScreenRegion.from_region(1, 1, 4, 0)),
        ("ScreenRegion.from_tuple((0, 0))", lambda: ScreenRegion.from_tuple((0, 0))),
        ("Pass.set_viewport([0, 0, 0, 4])", lambda: rpass.set_viewport([0, 0, 0, 4])),
        ("Pass.set_viewport([4, 0])", lambda: rpass.set_viewport([4, 0])),
        ("ScreenRegion((2**32 - 1, 0), (5, 5))", lambda: ScreenRegion((2**32 - 1, 0), (5, 5))),
        ("ScreenRegion.from_region(0, 2**32 - 3, 4, 4)", lambda: ScreenRegion.from_region(0, 2**32 - 3, 4, 4)),
        ("Pass.set_viewport([2**32 - 1, 0, 2, 2])", lambda: rpass.set_viewport([2**32 - 1, 0, 2, 2])),
    ):
        try:
            call()
        except InvalidInputError as e:
            print(f"{label} raised InvalidInputError: {e}")
        else:
            raise AssertionError(f"{label} should raise InvalidInputError")
    rpass.set_viewport([0, 0, 1, 1])
    assert ScreenRegion((2**32 - 6, 0), (5, 5)).max_x == 2**32 - 1
    print("Regions refuse a side of 0 and a far edge past 2**32 - 1")


def check_labels(renderer):
    """Number errors name the parameter, and the item for a list, as JavaScript's do."""
    def message(call):
        try:
            call()
        except FragmentColorError as e:
            return str(e)
        raise AssertionError("the call should raise FragmentColorError")

    assert "Mesh.set_indices: item 1" in message(lambda: Mesh().set_indices([0, -1, 2]))
    assert "Pass.set_compute_dispatch: y" in message(lambda: Pass("p").set_compute_dispatch(1, -1, 1))
    assert "Size: height" in message(lambda: Size(4, -1))
    assert "Pass.set_viewport: item 2" in message(lambda: Pass("p").set_viewport([0, 0, -4, 4]))
    assert "Pass.set_clear_color: item 1" in message(lambda: Pass("p").set_clear_color([0, 10**400, 0]))
    assert "Pass.set_clear_color takes 3 or 4 numbers, got 5" in message(lambda: Pass("p").set_clear_color([0.5] * 5))
    assert "Light.point: position takes 3 numbers, got 2" in message(lambda: Light.point([0, 1], [1, 1, 1]))
    assert "Renderer.read_texture: texture_id" in message(lambda: renderer.read_texture(-1))
    texture = renderer.create_texture(bytes(64), size=[4, 4])
    for count in (3, 5):
        raises_invalid(f"Texture.write_region with {count} numbers", lambda: texture.write_region(bytes(4), [0] * count))
    print("Number errors name their parameter")


def raises_invalid(label, call):
    try:
        call()
    except InvalidInputError as e:
        print(f"{label} raised InvalidInputError: {e}")
        return
    raise AssertionError(f"{label} should raise InvalidInputError")


def raises_type(label, call):
    try:
        call()
    except TypeError as e:
        print(f"{label} raised TypeError: {e}")
        return
    raise AssertionError(f"{label} should raise TypeError")


def check_colors(renderer):
    """Every color setter takes a number sequence, refuses bytes and bools, and refuses a channel past float range."""
    rpass, light, scene, material = Pass("colors"), Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), Scene(), Material.pbr()
    setters = {
        "Pass.set_clear_color": (rpass.set_clear_color, 4),
        "Light.set_color": (light.set_color, 3),
        "Scene.ambient": (scene.ambient, 3),
        "Material.base_color": (material.base_color, 4),
        "Material.emissive": (material.emissive, 3),
        "Light.point": (lambda color: Light.point([0.0, 0.0, 0.0], color), 3),
        "Light.directional": (lambda color: Light.directional([0.0, -1.0, 0.0], color), 3),
        "Light.spot": (lambda color: Light.spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], color), 3),
    }
    for name, (setter, count) in setters.items():
        setter([0.5] * count)
        setter(tuple([0.5] * count))
        setter(np.full(count, 0.5, dtype=np.float32))
        for bytes_form in (bytes(count), bytearray(count), memoryview(bytes(count)), np.zeros(count, dtype=np.uint8)):
            raises_type(f"{name}({type(bytes_form).__name__})", lambda: setter(bytes_form))
        raises_type(f"{name} with a string channel", lambda: setter(["1"] + [0] * (count - 1)))
        raises_type(f"{name} with a bool channel", lambda: setter([True] + [0] * (count - 1)))
        raises_type(f"{name} with a NumPy bool channel", lambda: setter(np.ones(count, dtype=np.bool_)))
        raises_invalid(f"{name} with 10**400", lambda: setter([10**400] + [0] * (count - 1)))
        for bad in (1e39, float("nan"), float("inf"), float("-inf")):
            raises_invalid(f"{name} with {bad}", lambda: setter([bad] + [0] * (count - 1)))
        raises_invalid(f"{name} with too many channels", lambda: setter([0.5] * 5))
    # Storage texture usage reads as an int too.
    raises_invalid("create_storage_texture(usage=-1)",
                   lambda: renderer.create_storage_texture([4, 4], TextureFormat.Rgba8Unorm, usage=-1))
    raises_invalid("create_storage_texture(usage=10**400)",
                   lambda: renderer.create_storage_texture([4, 4], TextureFormat.Rgba8Unorm, usage=10**400))
    print("Color setters take number sequences and refuse bytes and out-of-range channels")


def check_vectors():
    """Every position, direction, axis and scale takes three numbers the way a color takes its channels."""
    light, model, camera = Light.point([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]), Model(Mesh(), Material.pbr()), Camera.perspective(1.0, 1.0, 0.1, 100.0)
    vectors = {
        "Light.point: position": lambda v: Light.point(v, [1.0, 1.0, 1.0]),
        "Light.directional: direction": lambda v: Light.directional(v, [1.0, 1.0, 1.0]),
        "Light.spot: position": lambda v: Light.spot(v, [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]),
        "Light.spot: direction": lambda v: Light.spot([0.0, 1.0, 0.0], v, [1.0, 1.0, 1.0]),
        "Light.set_position": lambda v: light.set_position(v),
        "Light.set_direction": lambda v: Light.spot([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0, 1.0]).set_direction(v),
        "Model.translate": lambda v: model.translate(v),
        "Model.scale": lambda v: model.scale(v),
        "Model.rotate: axis": lambda v: model.rotate(v, 0.5),
        "Camera.look_at: position": lambda v: camera.look_at(v, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        "Camera.look_at: target": lambda v: camera.look_at([0.0, 0.0, 5.0], v, [0.0, 1.0, 0.0]),
        "Camera.look_at: up": lambda v: camera.look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], v),
    }
    for name, apply in vectors.items():
        apply([1, 2, 3])
        apply((1.0, 2.0, 3.0))
        apply(np.array([1.0, 2.0, 3.0], dtype=np.float32))
        for bytes_form in (bytes(3), bytearray(3), memoryview(bytes(3)), np.zeros(3, dtype=np.uint8)):
            raises_type(f"{name}({type(bytes_form).__name__})", lambda: apply(bytes_form))
        raises_type(f"{name} with a bool", lambda: apply([True, 1, 0]))
        raises_type(f"{name} with a string", lambda: apply(["0", 1, 0]))
        raises_invalid(f"{name} with 2 numbers", lambda: apply([0.0, 1.0]))
        raises_invalid(f"{name} with 10**400", lambda: apply([10**400, 1, 0]))
        for bad in (1e39, float("nan"), float("inf"), float("-inf")):
            raises_invalid(f"{name} with {bad}", lambda: apply([bad, 1, 0]))
    assert light.set_position(np.array([1.0, 2.0, 3.0])).position() == [1.0, 2.0, 3.0]

    # A matrix is 16 numbers or 4 columns of 4, as a list, a tuple or an ndarray.
    identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    model.set_transform([2.0] + [0.0] * 15)
    assert model.transform()[0][0] == 2.0
    model.set_transform(identity)
    model.set_transform(np.eye(4, dtype=np.float32))
    model.set_transform(tuple(np.arange(16.0)))
    assert model.transform()[1][0] == 4.0, "a flat matrix fills column after column"
    for label, matrix in (("15 numbers", [0.0] * 15), ("3 columns", identity[:3]),
                          ("a column of 3", identity[:3] + [[0.0, 0.0, 1.0]]), ("10**400", [10**400] + [0.0] * 15)):
        raises_invalid(f"Model.set_transform with {label}", lambda: model.set_transform(matrix))
    for label, matrix in (("a string", ["1"] + [0.0] * 15), ("a bool", [True] + [0.0] * 15), ("bytes", bytes(16))):
        raises_type(f"Model.set_transform with {label}", lambda: model.set_transform(matrix))

    # A Quad corner takes two numbers and a vertex position 1 to 4, under the same rule.
    points = {
        "Quad: min": lambda v: Quad(v, [1.0, 1.0]),
        "Quad: max": lambda v: Quad([0.0, 0.0], v),
        "Vertex: position": lambda v: Vertex(v),
        "Vertex.pbr: position": lambda v: Vertex.pbr(v),
        "Mesh.add_vertex: vertex": lambda v: Mesh().add_vertex(v),
        "Mesh.add_vertices: vertex 0": lambda v: Mesh().add_vertices([v]),
        "Mesh.from_vertices: vertex 0": lambda v: Mesh.from_vertices([v]),
    }
    for name, apply in points.items():
        apply([0.5, 1])
        apply((0.5, 1.0))
        apply(np.array([0.5, 1.0], dtype=np.float32))
        for bytes_form in (bytes(2), bytearray(2), memoryview(bytes(2)), np.zeros(2, dtype=np.uint8)):
            raises_type(f"{name}({type(bytes_form).__name__})", lambda: apply(bytes_form))
        raises_type(f"{name} with a bool", lambda: apply([True, 1]))
        raises_type(f"{name} with a string", lambda: apply(["0", 1]))
        for bad in (10**400, 1e39, float("nan"), float("inf"), float("-inf")):
            try:
                apply([bad, 1.0])
            except InvalidInputError as e:
                assert str(e).startswith(name), f"{name} with {bad} raised {e}"
            else:
                raise AssertionError(f"{name} with {bad} should raise InvalidInputError")
    raises_invalid("Quad with 3 numbers", lambda: Quad([0.0, 0.0, 0.0], [1.0, 1.0]))
    raises_invalid("Vertex with 5 numbers", lambda: Vertex([0.0] * 5))
    raises_invalid("Vertex(nan)", lambda: Vertex(float("nan")))
    raises_type("Vertex(True)", lambda: Vertex(True))
    Vertex(0.5)
    Mesh().add_vertex(([0.0, 0.0, 0.0], [0.0, 1.0]))
    raises_invalid("Mesh.add_vertex((position with NaN, uv))",
                   lambda: Mesh().add_vertex(([float("nan"), 0.0, 0.0], [0.0, 1.0])))
    print("Vectors take three numbers and refuse bytes, bools and out-of-range items")


RESOLUTION_SHADER = """
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(resolution / 1000.0, 0., 1.); }
"""


def check_example_resize(renderer):
    """The examples' rendercanvas resize handler hands resize whole pixels from float event sizes."""
    import ast
    for path in ("examples/python/main.py", "examples/python/multiobject.py"):
        tree = ast.parse(Path(path).read_text())
        handler = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "handler")
        handler.decorator_list = []
        shader = Shader(RESOLUTION_SHADER)
        target = renderer.create_texture_target([8, 8])
        namespace = {"circle": shader, "circles": [shader], "target": target}
        exec(compile(ast.Module([handler], []), path, "exec"), namespace)
        namespace["handler"]({"event_type": "resize", "width": 400.0, "height": 300.0, "pixel_ratio": 1.5})
        size = target.size()
        assert (size.width, size.height) == (600, 450), f"{path} resized to {size.width}x{size.height}"
    print("The examples resize their target from float event sizes")


def check_clear_color():
    """A clear color keeps its numbers, raises InvalidInputError for NaN, and TypeError for a string."""
    rpass = Pass("clear")
    for label, color in (("NaN", [float("nan"), 0, 0]), ("infinity", [0, 0, float("inf"), 1]), ("two channels", [0, 0])):
        try:
            rpass.set_clear_color(color)
        except InvalidInputError as e:
            assert str(e).startswith("Pass.set_clear_color"), f"set_clear_color with {label} raised {e}"
        else:
            raise AssertionError(f"set_clear_color with {label} should raise InvalidInputError")
    try:
        rpass.set_clear_color(["1", 0, 0])
    except TypeError:
        pass
    else:
        raise AssertionError("set_clear_color with a string should raise TypeError")
    rpass.set_clear_color([4.0, -1.0, 0.25])
    print("Clear colors keep their numbers and refuse NaN")


def check_texture_inputs(renderer):
    """create_texture takes usage, bytearray, memoryview, tuples and Textures."""
    pixels = bytes(range(16))
    texture = renderer.create_texture(pixels, size=[2, 2], usage=TextureUsage.COPY_SRC)
    assert bytes(renderer.read_texture(texture.id())) == pixels
    assert texture.id() == texture.id() and isinstance(texture.id().id, int)
    for source in (bytearray(pixels), memoryview(pixels), tuple(pixels)):
        renderer.create_texture(source, size=[2, 2])
    assert renderer.create_texture(texture).id() == texture.id()
    try:
        Mipmap.build(bytes(32), TextureFormat.Rgba8UnormSrgb)
    except MalformedImageError as e:
        assert isinstance(e, FragmentColorError)
    else:
        raise AssertionError("undecodable bytes should raise MalformedImageError")
    print("create_texture takes every byte form, a Texture, and usage")


def check_invalid_textures(renderer):
    """Texture inputs fill the format they are given, or raise InvalidInputError."""
    gray = np.arange(16, dtype=np.uint8).reshape(4, 4)
    texture = renderer.create_texture(
        gray, format=TextureFormat.R8Unorm, mipmaps=False, usage=TextureUsage.COPY_SRC
    )
    assert bytes(renderer.read_texture(texture.id())) == gray.tobytes(), "R8Unorm read the gray bytes"

    def invalid(label, call):
        try:
            call()
        except InvalidInputError as e:
            print(f"{label} raised InvalidInputError: {e}")
            return
        raise AssertionError(f"{label} should raise InvalidInputError")

    invalid("a gray ndarray as Rgba32Float", lambda: renderer.create_texture(gray, format=TextureFormat.Rgba32Float))
    invalid("an ndarray with another size", lambda: renderer.create_texture(gray, size=[2, 8]))
    invalid("an ndarray with 5 channels", lambda: renderer.create_texture(np.zeros((2, 2, 5), np.uint8)))
    invalid("a 4D ndarray", lambda: renderer.create_texture(np.zeros((2, 2, 2, 4), np.uint8)))
    print("create_texture fits an ndarray to its format and size")

    storage = renderer.create_storage_texture([16, 16], TextureFormat.Rgba)
    patch = bytes(4 * 4 * 4)
    storage.write_region(patch, {"min_x": 4, "max_x": 8, "min_y": 0, "max_y": 4})
    invalid("reversed bounds", lambda: storage.write_region(patch, {"min_x": 8, "max_x": 4, "max_y": 4}))
    invalid("misspelled keys", lambda: storage.write_region(patch, {"x": 0, "w": 4, "h": 4}))
    invalid("both region forms", lambda: storage.write_region(patch, {"x": 0, "width": 4, "height": 4, "max_x": 4}))
    # 4 rows of 16 texels, the bytes a 16x4 region holds, so only the zero width is wrong.
    invalid("a zero width region", lambda: storage.write_region(bytes(16 * 4 * 4), [0, 0, 0, 4]))
    storage.write_region(bytes(16 * 16 * 4), None)
    print("write_region reads a dict region or raises InvalidInputError")

    invalid("a path with a size", lambda: renderer.create_texture("examples/assets/logo.png", size=[64, 64]))
    invalid("usage 32", lambda: renderer.create_texture(bytes(16), size=[2, 2], usage=32))
    invalid("storage usage 0", lambda: renderer.create_storage_texture([4, 4], TextureFormat.Rgba, usage=0))
    invalid("storage on Depth32Float", lambda: renderer.create_storage_texture([4, 4], TextureFormat.Depth32Float))
    invalid("raw bytes into Depth32Float", lambda: renderer.create_texture(bytes(64), size=[4, 4], format=TextureFormat.Depth32Float))
    invalid("extra raw bytes", lambda: renderer.create_texture(bytes(20), size=[2, 2]))
    print("create_texture and create_storage_texture refuse what wgpu would")


def check_scene_errors():
    """A failed Scene.load raises a FragmentColorError that is also a RuntimeError."""
    for kind in (FragmentColorError, RuntimeError, SceneLoadError):
        try:
            Scene.load("no/such/file.glb", cameras=False, lights=False)
        except kind:
            continue
        raise AssertionError(f"a missing scene file should raise {kind.__name__}")
    for bad in (42, None):
        try:
            Shader(bad)
        except TypeError:
            continue
        raise AssertionError(f"Shader({bad!r}) should raise TypeError")
    print("Scene.load and Shader raise the documented errors")


def check_stub():
    """The stub leaves out the binding plumbing that dispatch no longer calls."""
    stub = (Path(__file__).parent / "fragmentcolor" / "fragmentcolor.pyi").read_text()
    for plumbing in ("def passes(", "def renderable_type("):
        assert plumbing not in stub, f"the stub still lists {plumbing}"
    print("The stub has no passes or renderable_type")


# Green when `id` arrives as the u32 16777217 and `cell` as the vec2<i32> (-1, 2), red otherwise.
# The vertex stage compares the values, so every pixel is exactly green or red.
INTEGER_ATTRIBUTES_SHADER = """
struct VOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) @interpolate(flat) ok: u32,
};
@vertex fn vs_main(
  @location(0) position: vec2<f32>,
  @location(1) id: u32,
  @location(2) cell: vec2<i32>,
) -> VOut {
  var out: VOut;
  out.pos = vec4f(position, 0., 1.);
  out.ok = select(0u, 1u, id == 16777217u && all(cell == vec2<i32>(-1, 2)));
  return out;
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  if (v.ok == 1u) { return vec4f(0., 1., 0., 1.); }
  return vec4f(1., 0., 0., 1.);
}
"""

FULLSCREEN = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]


def check_vertex_values(renderer):
    """Plain numbers on a Vertex or Instance take the type the shader declares."""
    def assert_green(label, shader):
        target = renderer.create_texture_target([8, 8])
        renderer.render(Pass.from_shader(label, shader), target)
        pixel = list(target.get_image()[0, 0])
        assert pixel == [0, 255, 0, 255], f"{label}: pixel {pixel}, want green"

    # 2^24 + 1 only survives as an integer, so a float transport would draw red.
    vertices = Mesh()
    for corner in FULLSCREEN:
        vertices.add_vertex(Vertex(corner).set("id", 16777217).set("cell", np.array([-1, 2], dtype=np.int32)))
    vertex_shader = Shader(INTEGER_ATTRIBUTES_SHADER)
    vertex_shader.add_mesh(vertices)
    assert_green("vertex numbers", vertex_shader)

    instanced = Mesh()
    for corner in FULLSCREEN:
        instanced.add_vertex(Vertex(corner).set("cell", (-1, 2)))
    instanced.add_instance({"id": 16777217})
    instance_shader = Shader(INTEGER_ATTRIBUTES_SHADER)
    instance_shader.add_mesh(instanced)
    assert_green("instance numbers", instance_shader)

    def rejects(label, mesh, needle):
        try:
            Shader(INTEGER_ATTRIBUTES_SHADER).add_mesh(mesh)
        except FragmentColorError as e:
            assert needle in str(e), f"{label}: {e!r} does not name {needle!r}"
            print(f"{label} rejected: {e}")
            return
        raise AssertionError(f"{label} should raise FragmentColorError")

    negative = Mesh()
    negative.add_vertex(Vertex([0.0, 0.0]).set("id", 7).set("cell", [0, 0]))
    negative.add_vertex(Vertex([1.0, 0.0]).set("id", -1).set("cell", [0, 0]))
    rejects("-1 for a u32", negative, "expected u32, got -1 at vertex 1")
    fraction = Mesh()
    fraction.add_vertex(Vertex([0.0, 0.0]).set("id", 7).set("cell", [2.5, 0]))
    rejects("2.5 for an i32", fraction, "expected vec2<i32>, got 2.5")

    for label, values in (("five numbers", [1, 2, 3, 4, 5]), ("no numbers", [])):
        try:
            Instance().set("offset", values)
        except FragmentColorError as e:
            print(f"Instance.set with {label} raised: {e}")
            continue
        raise AssertionError(f"Instance.set with {label} should raise FragmentColorError")
    print("Vertex and Instance numbers take the shader types")


INSTANCED_SHADER = """
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) tint: vec4<f32> };
@vertex fn vs_main(
  @location(0) position: vec2<f32>,
  @location(1) offset: vec2<f32>,
  @location(2) tint: vec4<f32>,
) -> VOut {
  var out: VOut;
  out.pos = vec4f(position + offset, 0., 1.);
  out.tint = tint;
  return out;
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.tint; }
"""


def check_mesh_inputs():
    """The Mesh, Vertex, and Instance methods take the forms their stub lists."""
    mesh = Mesh.from_vertices([[0.0, 0.0], (1.0, 0.0), Vertex([1.0, 1.0])])
    mesh.add_vertices([((0.0, 1.0, 0.0), (0.0, 1.0)), ((1.0, 1.0, 0.0), (1.0, 1.0), (1.0, 1.0, 1.0, 1.0))])
    mesh.add_vertex(Vertex(Vertex([0.5, 0.5])))
    mesh.set_indices(np.array([0, 1, 2], dtype=np.uint32))

    def rejects(label, call):
        try:
            call()
        except TypeError as e:
            print(f"rejected {label}: {str(e).splitlines()[0]}")
            return
        raise AssertionError(f"{label} should raise TypeError")

    raises_invalid("a five-number position", lambda: Vertex([1, 2, 3, 4, 5]))
    rejects("a string attribute", lambda: Vertex([0, 0]).set("w", "x"))
    rejects("a number as an instance", lambda: Mesh().add_instance(3.0))
    rejects("a string instance attribute", lambda: Mesh().add_instance({"offset": "x"}))

    # A dict instance sets its attributes in insertion order, so locations follow it.
    instanced = Mesh()
    instanced.add_vertex([0.0, 0.0])
    instanced.add_instances([
        {"offset": [0.1, 0.2], "tint": [1.0, 0.0, 0.0, 1.0]},
        Instance().set("offset", [0.3, 0.4]).set("tint", [0.0, 1.0, 0.0, 1.0]),
    ])
    Shader(INSTANCED_SHADER).add_mesh(instanced)
    print("Mesh, Vertex, and Instance take their typed inputs")


COLOR_SHADER = """
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> };
@group(0) @binding(0) var<uniform> gain: f32;
@vertex fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VOut {
  return VOut(vec4<f32>(position, 0.0, 1.0), color * gain);
}
@fragment fn fs_main(v: VOut) -> @location(0) vec4<f32> { return v.color; }
"""


def check_mesh_draws(renderer):
    """One shader draws meshes with other layouts, a second Renderer draws them too,
    and render and add_mesh refuse the mesh data they can't draw."""
    def column(left, right):
        return [[left, -1.0], [right, -1.0], [right, 1.0], [left, -1.0], [right, 1.0], [left, 1.0]]

    def refuses(label, words, call):
        try:
            call()
        except FragmentColorError as e:
            assert words in str(e), f"{label}: {e!r} does not name {words!r}"
            print(f"{label} raised FragmentColorError: {e}")
            return
        raise AssertionError(f"{label} should raise FragmentColorError")

    shader = Shader(COLOR_SHADER)
    shader.set("gain", 1.0)
    painted = Mesh()
    for corner in column(-1.0, 0.0):
        painted.add_vertex(Vertex(corner).set("color", [1.0, 0.0, 0.0, 1.0]))
    tinted = Mesh()
    tinted.add_vertices(column(0.0, 1.0))
    tinted.add_instance({"color": [0.0, 0.0, 1.0, 1.0]})
    shader.add_mesh(painted)
    shader.add_mesh(tinted)

    for label, painter in (("one Renderer", renderer), ("a second Renderer", Renderer())):
        target = painter.create_texture_target([8, 2])
        painter.render(shader, target)
        image = target.get_image()
        assert list(image[0, 1]) == [255, 0, 0, 255], f"{label}: vertex color column {list(image[0, 1])}"
        assert list(image[0, 6]) == [0, 0, 255, 255], f"{label}: instance color column {list(image[0, 6])}"

    target = renderer.create_texture_target([8, 2])
    painted.set_indices([0, 1, 7])
    refuses("an index past the last vertex", "points past the last of 6 vertices",
            lambda: renderer.render(shader, target))
    painted.set_indices([0, 1, 2, 0])
    refuses("a partial triangle", "multiple of 3 indices", lambda: renderer.render(shader, target))
    painted.clear_indices()
    tinted.add_instance(Instance())
    refuses("an instance without the color the shader reads", "got no value at instance 1",
            lambda: renderer.render(shader, target))
    tinted.clear_instances()
    tinted.add_instance({"color": [0.0, 0.0, 1.0, 1.0]})
    named = Mesh()
    named.add_vertex(Vertex([0.0, 0.0]).set("position", [5.0, 5.0]).set("color", [1.0, 1.0, 1.0, 1.0]))
    refuses("a position property", "property named 'position'", lambda: Shader(COLOR_SHADER).add_mesh(named))
    renderer.render(shader, target)
    print("One shader draws meshes with other layouts, and refuses mesh data it can't draw")


def run():
    """Simple smoke tests before running full examples"""

    # Tag this process for GPU error context
    os.environ["FC_RUNNER"] = "python"
    os.environ["FC_CURRENT_TEST"] = "platforms.python.healthcheck"
    take_gpu_errors()

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
    assert image.shape[0] == 64  # height
    assert image.shape[1] == 32  # width
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
    check_image_layout(renderer)
    check_tight_writes(renderer)
    check_error_types(renderer)
    check_vertex_values(renderer)
    check_mesh_inputs()
    check_mesh_draws(renderer)
    check_require_graph(renderer)
    check_pass_targets(renderer)
    check_value_types()
    check_number_errors(renderer)
    check_empty_regions()
    check_labels(renderer)
    check_colors(renderer)
    check_vectors()
    check_example_resize(renderer)
    check_clear_color()
    check_texture_inputs(renderer)
    check_invalid_textures(renderer)
    check_scene_errors()
    check_stub()

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

    errors = take_gpu_errors()
    assert not errors, f"GPU error: {errors}"


if __name__ == "__main__":
    run()

    # Auto-generated: run all extracted & translated examples from docs/api
    run_all()
