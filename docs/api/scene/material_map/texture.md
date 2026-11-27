# MaterialMap::texture

Set the texture the map samples, and return the map. The material's shader
reads it under the map's name, such as `base_color_map`, through the UV set
[`uv`](https://fragmentcolor.org/api/scene/materialmap#uv) picks and the
[`transform`](https://fragmentcolor.org/api/scene/materialmap#transform) the
map applies.

Accepts any `Into<TextureInput>`: another map, or the inputs [`Renderer::create_texture`](https://fragmentcolor.org/api/core/renderer#create_texture) takes:

- Another [`&MaterialMap`](https://fragmentcolor.org/api/scene/materialmap), such as a loaded glTF material's `&pbr.base_color_map()`. The map samples the texture the given map samples, under the same `TextureId`. A `Texture` there binds here as it is. A source stays one source, which each `Renderer` uploads once for both maps, and a `Renderer` that loaded it for the given map draws it for this one at once. One upload can't serve two color spaces, so a map that reads its source in another one, such as a normal map given a base color map, keeps the same source and uploads a copy of its own in its own space. A format the caller picked stays, as a `Texture`'s does. The map keeps its own UV set and transform, since `texture` sets only the texture and the sampler that comes with it. Either map set again later takes a texture of its own and leaves the other as it was. A map with no texture, which draws its 1×1 default, has none to share, and the call refuses it, as it refuses options beside a map.
- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture). The map stores the texture's ID immediately and the GPU texture stays Arc-shared across every material that points at it. Passing the same `&shared_texture` to the maps of N materials produces one GPU upload and N shader-uniform references: the cheap way to reuse one texture across a scene full of objects without paying for N texture allocations. The texture belongs to the `Renderer` that created it, and another `Renderer` refuses to draw it. It keeps the options it was created with, and the call refuses options beside it.
- Encoded bytes, raw pixels with their size as `(bytes, [w, h])`, bytes with a format as `(bytes, format)`, bytes with every option as `(bytes, options)`, a path or a URL, alone or as `(path, format)` or `(path, options)`, or a [`Mipmap`](https://fragmentcolor.org/api/texture/mipmap), alone or as `(chain, options)`. The material keeps this source in the map. Each `Renderer` that draws the material uploads a copy of its own at [`Renderer::load`](https://fragmentcolor.org/api/core/renderer#load), or on native targets at its first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render), and frees it when the `Renderer` or the material drops. A map set again draws its old copy until the `Renderer` loads the new one.

To share a map across Renderers, pass bytes, a path, a URL or a `Mipmap`.

Each language takes another map as `texture(otherMap)`, or `texture(other_map)`
in Python, with no options, and spells the other inputs the way its
`create_texture` does:

- **JavaScript:** `texture(input, options?)`, where `options` is a size such as `[w, h]`, a `TextureFormat`, or `{ size?, format?, mipmaps?, sampler?, usage? }`.
- **Python:** `texture(input, size=None, format=None, mipmaps=None, sampler=None, usage=None)`. A NumPy array brings its own size, and a `size` that differs from it raises `InvalidInputError`.
- **Swift and Kotlin:** `texture(map)`, `texture(texture)`, `texture(bytes)`, `texture(bytes, format)`, `texture(bytes, [w, h])`, `texture(bytes, size)`, `texture(bytes, options)`, `texture(path)`, `texture(path, format)`, `texture(path, options)`, `texture(chain)` and `texture(chain, options)`. Bytes are `Data`, `[UInt8]` or `[Int]` in Swift and `ByteArray` or `List<Int>` in Kotlin, in every bytes form. The bytes forms without `options` take an optional `usage`. An `Int` outside 0 to 255 throws, and so does a `[w, h]` that isn't 2 or 3 numbers.

A source with no format of its own reads in the map's color space: sRGB in
the base color and emissive maps, as glTF's color textures do, and linear in
the maps that hold data, such as normals. A `Mipmap` or a KTX2 container
keeps the format it stores, and takes that sRGB reading only where its bytes
read the same in it, so a float chain or container stays linear. A format you
pass yourself is the one it uploads in, and a `Mipmap` or KTX2 container
whose bytes can't be read in it fails the `load` with an `InvalidInput` error
that names `load`.

Unset, a map draws a 1×1 default the renderer hands out lazily, which leaves
the map's factor as it is, so `PbrMaterial::new()` renders correctly without
a texture. Each map's page names its default.

## Memory

A map holds its source for as long as the material keeps it. That is the file's
bytes for an encoded PNG or JPEG, a glTF image inside its file included, the
pixels for raw bytes or a NumPy array, the container for KTX2, and the chain
for a `Mipmap`. A path or
a URL holds only its string, and each `Renderer` reads or fetches it again.
`renderer.create_texture` keeps nothing on the CPU, and the texture it returns
draws in that one `Renderer`. A map that takes another map's texture holds the
same source, so it adds nothing to that.

## Errors

The call checks its input at once, and a call it refuses leaves the map as it
was. Options beside a `Texture` return `MaterialError::Texture` with an
`InvalidInput` texture error that names the call, such as
`PbrMaterial::base_color_map().texture`, and so does an input that holds no
data: Rust's `TextureData::Empty`, or an empty byte list, path or URL in any
language. So do options beside a map, and a map with no texture, whose
error names that map too, as in
`ToonMaterial::base_color_map().texture: PbrMaterial::normal_map() has no texture to share`.

The bindings read their arguments before the setter runs and throw for one
they can't use: a size that isn't 2 or 3 whole numbers, options of the wrong
type or shape, options beside a `Texture` or a map, and in Python a `size` that
differs from a NumPy array's own. Python raises `TypeError` for an argument
of the wrong type and `InvalidInputError` for the rest. JavaScript throws an `Error`
of the same name. Swift throws `FragmentColorError`, and each call needs
`try`, while Kotlin throws `FragmentColorException`.

A source that fails to upload (file not found, decode error, unsupported
format) fails [`Renderer::load(&material).await`](https://fragmentcolor.org/api/core/renderer#load),
or the native render that runs it. That `Renderer` keeps no copy and tries
again at each later call, until the source is fixed or replaced, while the
other maps upload.

On the web, `render` uploads nothing, so await `load` in each `Renderer`
before its first frame. Until then the map draws its 1×1 default. On native, only a map set on another thread during a `render`
meets this, for that frame.

For deterministic error handling, pre-build the texture eagerly:
`renderer.create_texture(path).await?` returns a `Texture` that has already
uploaded, so no `load` can fail on it later, and
`texture(&texture)` takes an Arc-shared reference. The
lazy path stays useful when the URL / path resolves at render time but the
caller doesn't want to await yet.

## Example: sharing one texture across many materials

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{PbrMaterial, Renderer, ToonMaterial};

let renderer = Renderer::new();
let albedo_bytes: Vec<u8> = vec![
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
];
let albedo = renderer.create_texture((albedo_bytes, [2, 2])).await?;

// Every material whose map points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle to N maps costs one upload and N
// shader-uniform references.
let material = PbrMaterial::new();
let map = material.base_color_map().texture(&albedo)?;

// Raw pixels with their size upload when the scene loads.
let tint: Vec<u8> = vec![255, 128, 0, 255];
let tinted = PbrMaterial::new();
let tinted_map = tinted.base_color_map().texture((tint, [1, 1]))?;

// A toon material samples the tinted texture too, which each Renderer
// uploads once for both maps.
let toon = ToonMaterial::new();
let toon_map = toon.base_color_map().texture(&tinted.base_color_map())?;
# let _ = (map, tinted_map, toon_map);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
