# Material::base_color_texture

Bind a texture to the `base_color_map` slot. The default PBR shader samples it in `fs_main` and multiplies by the factor. The albedo for each fragment is `material.base_color * textureSample(base_color_map, pbr_sampler, uv) * vertex_color`, where `uv` is `slot_uv(material.uv[0], in.uv0, in.uv1)`: the UV set [`texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv) picks for the slot, through the slot's [`texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform).

Accepts any `Into<TextureInput>`, the inputs [`Renderer::create_texture`](https://fragmentcolor.org/api/core/renderer#create_texture) takes:

- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture). The setter stores the texture's ID immediately and the GPU texture stays Arc-shared across every Material that points at it. Passing the same `&shared_texture` to N Materials produces one GPU upload and N shader-uniform references: the cheap way to reuse one texture map across a scene full of objects without paying for N texture allocations. The texture belongs to the `Renderer` that created it, and another `Renderer` refuses to draw it. It keeps the options it was created with, so JavaScript and Python refuse options beside it with an `InvalidInputError`.
- Encoded bytes, raw pixels with their size as `(bytes, [w, h])`, bytes with a format as `(bytes, format)`, bytes with every option as `(bytes, options)`, a path or a URL, alone or as `(path, format)` or `(path, options)`, or a [`Mipmap`](https://fragmentcolor.org/api/texture/mipmap), alone or as `(chain, options)`. The Material keeps this source as a map. Each `Renderer` that draws the Material uploads a copy of its own at [`Renderer::load`](https://fragmentcolor.org/api/core/renderer#load), or on native targets at its first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render), and frees it when the `Renderer` or the Material drops. A map set again draws its old copy until the `Renderer` loads the new one.

To share a map across Renderers, pass bytes, a path, a URL or a `Mipmap`.

Each language spells these inputs the way its `create_texture` does, and the other four texture setters take the same ones:

- **JavaScript:** `baseColorTexture(input, options?)`, where `options` is a size such as `[w, h]`, a `TextureFormat`, or `{ size?, format?, mipmaps?, sampler?, usage? }`.
- **Python:** `base_color_texture(input, size=None, format=None, mipmaps=None, sampler=None, usage=None)`. A NumPy array brings its own size, and a `size` that differs from it raises `InvalidInputError`.
- **Swift and Kotlin:** `baseColorTexture(texture)`, `baseColorTexture(bytes)`, `baseColorTexture(bytes, format)`, `baseColorTexture(bytes, [w, h])`, `baseColorTexture(bytes, size)`, `baseColorTexture(bytes, options)`, `baseColorTexture(path)`, `baseColorTexture(path, format)`, `baseColorTexture(path, options)`, `baseColorTexture(chain)` and `baseColorTexture(chain, options)`. Bytes are `Data`, `[UInt8]` or `[Int]` in Swift and `ByteArray` or `List<Int>` in Kotlin, in every bytes form. The bytes forms without `options` take an optional `usage`. An `Int` outside 0 to 255 throws, and so does a `[w, h]` that isn't 2 or 3 numbers.

Unset, this slot resolves to a 1×1 white default the renderer hands out
lazily, so calling `Material::pbr()` without binding a
texture renders correctly under the factor alone.

## Memory

A map holds its source for as long as the slot keeps it. That is the file's
bytes for an encoded PNG or JPEG, the pixels for raw bytes, a NumPy array or
a glTF image, the container for KTX2, and the chain for a `Mipmap`. A path or
a URL holds only its string, and each `Renderer` reads or fetches it again.
`renderer.create_texture` keeps nothing on the CPU, and the texture it returns
draws in that one `Renderer`.

## Errors

In Rust the setter can't fail at the call. The bindings read their arguments
there and throw for one they can't use: a size that isn't 2 or 3 whole numbers,
options of the wrong type or shape, options beside a `Texture`, and in Python a
`size` that differs from a NumPy array's own. Python raises `TypeError` for an
argument of the wrong type and `InvalidInputError` for the rest, JavaScript throws
an `Error` of the same name, and Swift and Kotlin throw `FragmentColorError` and
`FragmentColorException`.

The setter checks the slot at once. A slot the shader doesn't declare as a
texture, or an empty input, logs a warning and leaves the Material as it was.

A source that fails to upload (file not found, decode error, unsupported
format) fails [`Renderer::load(&material).await`](https://fragmentcolor.org/api/core/renderer#load),
or the native render that runs it. That `Renderer` keeps no copy and tries
again at each later call, until the source is fixed or replaced, while the
other maps upload.

On the web, `render` uploads nothing, so await `load` in each `Renderer`
before its first frame. Until then a built-in PBR slot draws its 1×1 default,
and a custom shader's slot fails the render with a `BindingError` that asks
for `load`.

For deterministic error handling, pre-build the texture eagerly:
`renderer.create_texture(path).await?` returns a `Texture` that can't
fail at setter time, and `Material::base_color_texture(&texture)` takes
an Arc-shared reference. The lazy path stays useful when the URL /
path resolves at render time but the caller doesn't want to await yet.

## Example: sharing one texture across many Materials

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let albedo_bytes: Vec<u8> = vec![
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
];
let albedo = renderer.create_texture((albedo_bytes, [2, 2])).await?;

// Every Material that points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle into N Material instances costs one
// upload and N shader-uniform references.
let material = Material::pbr().base_color_texture(&albedo);

// Raw pixels with their size upload when the scene loads.
let tint: Vec<u8> = vec![255, 128, 0, 255];
let tinted = Material::pbr().base_color_texture((tint, [1, 1]));
# let _ = (material, tinted);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
