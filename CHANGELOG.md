# Changelog

## 0.12.7 (unreleased)

### Renderer and meshes

- `Shader.add_mesh` fails when a later vertex or instance lacks an attribute the shader reads, and the error names the attribute and the index: `color: expected vec4<f32>, got no value at vertex 3`. Only the first vertex and instance were checked, and a later one drew with zeros.
- `Material::pbr` writes light the target can show. On the `Rgba8Unorm` and `Bgra8Unorm` targets FragmentColor creates, which hold display values, it encodes as sRGB, so PBR scenes, docs images and goldens drawn there get brighter: they rendered dark and harsh. An sRGB target encodes on write and a float target keeps linear light, so both get linear light, and nothing encodes twice. The renderer sets the shader's `override ENCODE_SRGB: bool` from the target's format, and a custom shader that declares it gets the same value.
- The renderer releases a texture when the last thing holding it drops: every `Texture` handle, the `TextureTarget` that owns it, and each shader uniform or pass target that binds it. It kept every texture until `unregister_texture`, so a demo that made a new texture on each resize leaked the old ones.
- A pass that draws `Model`s and sets no depth target draws with a depth buffer, so overlapping scenes now draw with depth: the nearest surface shows whatever order the models were added in, where a far model added last covered a near one. The buffer matches the pass's color attachment in size and sample count. The renderer reuses it from frame to frame and frees it a few renders after the target changes size. The shaders in that pass that draw no model, such as a fullscreen backdrop or an overlay, ignore depth and draw in order, so a backdrop drawn first stays behind the models. `Pass.set_depth_target` still sets a pass's own depth texture, and passes without models, such as 2D and fullscreen passes, draw without depth.
- PBR materials light a `Model` scaled differently along each axis, or mirrored, the way its surfaces face. The registry's `mesh_transform_normal` uses the inverse transpose of the model matrix, where the plain matrix tilted normals toward the stretched axis.

### Textures

- `Texture.resize(size)` gives a texture a pass draws into a new size, in Rust, JavaScript, Python, Swift and Kotlin. Every clone of the handle follows, and so do the shader uniforms and pass targets that hold it, so a depth texture can follow a window without being created again. A new size starts with blank pixels. A texture without `RENDER_ATTACHMENT` usage returns `TextureError::FixedSize`, and a width or height of zero returns `TextureError::InvalidInput`. Both keep the old size.
- A `Texture` from `TextureTarget.texture()`, and every clone of a `TextureTarget`, follow `TextureTarget.resize`. They kept the old size, and `write` and `get_image` reached the old texture.
- `create_texture` decodes an HDR file into `Rgba16Float`, and a decoded image fills an explicit `Rgba16Float` or `Rgba32Float` format. Each uploaded RGBA8 bytes into a row four or eight times as wide and raised a GPU validation error, leaving the texture blank.
- A decoded image in `Bgra8Unorm` stores blue first in `create_texture` and `Mipmap.build`, so its red and blue no longer swap. A JavaScript `ImageData` or canvas does the same.
- `Texture.write` rebuilds the mip chain that `create_texture` builds by default, so a texture sampled far away shows the new image instead of the first one.
- `create_texture` reads an image's size from its header and checks it against the device before the pixels decode, so an image too large to upload takes no memory.
- Creating, writing and reading back a texture stays clear of GPU validation errors. The checks run before any wgpu call and return `TextureError::InvalidInput`, the error `resize` returns: a usage bit outside the five `TextureUsage` flags, a usage the format can't take on the device, such as `STORAGE_BINDING` on `Depth32Float`, a depth texture with layers, pixels for a depth texture, and a write or readback of a block-compressed KTX2 texture. A readback that needs a buffer past the device's largest, or a depth readback on GLES or WebGL2, returns `TextureError::Readback`.

### JavaScript

- `createStorageTexture`, `Mipmap.build`, `Material.alphaMode` and `Mipmap.level` throw an `Error` for a format, mode or index that isn't a whole number of the right kind. `undefined`, `NaN` and strings read as `R8Unorm` or `Opaque`, and fractions truncated, so `level(1.5)` returned level 1.
- `createTexture` and the Material texture setters read an options object by its keys. An object with `width` and `height` beside `format` or `usage` read as a bare size and dropped the rest. It now throws and asks for the size under `size`.
- A numeric `compare` in sampler options must be a whole number from 1 to 8, so `1.5` throws instead of reading as `Never`.
- `Vertex`, `Instance`, `Quad` and `TextureId` carry the `__fc_kind` brand the other classes have. Minifiers rename classes, and the checks that passed these objects into the library fell back to the class name, so a production build failed where a development build worked: `mesh.addVertices` rejected its `Vertex` objects. The checks read only the brand now, and a build fails when a type that crosses into the library has none. A site that keeps class names in its build to work around this, as the FragmentColor website does with `keepNames` in `docs/website/astro.config.mjs`, can drop that once it loads 0.12.7.

### Python

- Draws fire with rendercanvas 2.7, where `canvas.request_draw` never called the draw function and `examples/python/main.py` showed nothing. `Renderer.create_target` registers a rendercanvas context on the canvas, and `render` presents each window frame itself, from a draw function or a timer.
- A hidden or minimized window raises `OccludedSurfaceError`, a subclass of `FragmentColorError`, so a render loop can skip the frame without reading the message. Rust reports it as `SurfaceError::Occluded`.
- `create_texture` with a NumPy array honors `format`. An array whose channels match the format's bytes per pixel uploads as it is, so a gray `(H, W)` array fills `R8Unorm`, and 1 or 3 channels gain an opaque alpha for an 8-bit RGBA or BGRA format. The array was always expanded to RGBA8 first, so `R8Unorm` uploaded the expanded bytes. A side past `u32` is refused before the bytes are collected, where it truncated to a small texture.
- `size()` is a method on `Texture`, `TextureTarget` and `WindowTarget`, as in every other language and like every other Python getter. `TextureTarget.size` and `Texture.size` were properties, and `WindowTarget.size()` the one method among them.

### Swift

- `TextureTarget.resize([w, h])` and `WindowTarget.resize([w, h])` throw `FragmentColorError` for an array that isn't 2 or 3 numbers long, where they stopped the app.
- The `Camera`, `Light` and `Material` setters that return their receiver, such as `lookAt`, `setAspect` and `setDirection`, are `@discardableResult`, as `Scene` and `Pass` were. A per-frame update no longer needs `_ =` in front.

### Breaking and behavior changes

- `get_image` on a target returns an error where it returned an empty array. A `WindowTarget` always does, because a frame on screen can't be read back, and a `TextureTarget` does when its readback fails or times out. JavaScript throws an `Error`, Python and Swift throw `FragmentColorError`, Kotlin throws `FragmentColorException`, and Rust's `Target::get_image` returns `Result<Vec<u8>, TextureError>`.
- The raw uniffi `createTextureTarget` and `createDepthTexture` take a `Size`, so Swift's `createTextureTarget(width:height:)` and Kotlin's `createTextureTarget(32u, 64u)` are gone. The `[w, h]` and `Size` overloads stay and now carry the depth to the checks the other languages run, where they dropped it and a depth of 2 built a flat target.
- Overlapping models draw with depth. A model behind another stays hidden where, added last, it drew on top.
- In a pass that draws models, a shader that draws none, such as a fullscreen backdrop, ignores the depth target the pass sets, where it tested and wrote it and could hide every model.
- Texture ids are unique across renderers, so a second renderer's first texture no longer gets id 1.
- `resize` on a target can fail. Rust's `Target::resize` returns `Result<(), TextureError>`, JavaScript throws an `Error`, Python raises `FragmentColorError`, Swift calls need `try`, and Kotlin calls can throw `FragmentColorException`. A width or height of zero on a `TextureTarget` is an error that keeps its size. A `WindowTarget` accepts a width or height of zero, such as the 0×0 a minimized window reports, without an error: it keeps its last size and draws again once the window has one, where it handed the surface a size of zero. A side past the device limit is an error on either target. `App::resize` and `App::resize_target` return the same `Result`.
- `create_storage_texture` and `create_texture` check a texture's size against the device's limits and return the error `resize` returns, `TextureError::InvalidInput` inside `RendererError::TextureError` in Rust, where a size past them reached a wgpu validation error. That covers an explicit size, a decoded image, a prepared mip chain and a KTX2 file. A 2D texture takes each side up to the 2D limit, a 3D texture each side and its depth up to the 3D limit, and a side or depth of zero is refused. A compressed KTX2 file whose size isn't a whole number of blocks is refused the same way.
- Raw pixels for a 3D texture upload every slice. `create_texture` kept one mip level only for 2D, so the mipmap path wrote the first slice of a 3D texture and left the rest blank.
- `create_target`, `create_texture_target` and `create_depth_texture` check the size against the device limit and return the error `resize` returns, where a size past it reached a wgpu validation error. In Rust that is `TextureError::InvalidInput` inside the new `InitializationError::TextureError` variant. Swift and Kotlin get the `Render` case of `FragmentColorError` from these calls, as from `resize`, where creation errors were `Init`. A texture target or depth texture refuses a width or height of zero and a depth other than 1, and a window with no area yet still gets one pixel on that side.
- Resizing a `TextureTarget` to its current size keeps its pixels.
- A `TextureId` on its own doesn't keep its texture. Once the last handle, shader uniform and pass target holding a texture drop, `read_texture` and `unregister_texture` report its id as not found.
- Python reads sizes with a call: `texture.size.width` from the 0.12.6 notes is now `texture.size().width`, and `target.size` is `target.size()` on a texture target.
- `RendererError::MsaaViewMissing` is gone. No code returned it.
- Texture usage masks take the five `TextureUsage` flags only. `create_texture` and `create_storage_texture` refuse any other bit, and `create_storage_texture` refuses an empty mask, with `TextureError::InvalidInput`. wgpu-only bits reached wgpu as a validation error, and unknown ones were dropped.
- `create_storage_texture` and `create_texture` check the usage and dimension the device allows for the format, so a storage texture in `Depth32Float`, `Rgba8UnormSrgb` or `Bgra8Unorm`, and a `Depth32Float` texture with a depth past 1, return `TextureError::InvalidInput`. A format that needs a missing wgpu feature returns `TextureError::UnsupportedFormatForUsage` for any usage, where only `TEXTURE_BINDING` checked it.
- Raw bytes into a depth texture, from `create_texture`, a `create_storage_texture` seed or `Texture.write`, return `TextureError::InvalidInput`. Draw into a depth texture instead.
- `Texture.write`, `write_region`, `get_image` and `read_texture` on a block-compressed texture from KTX2 return `TextureError::InvalidInput`.
- A decoded image with an integer format override, such as `Rgba32Uint`, returns `TextureError::InvalidInput`. Pass raw bytes with a size for an integer texture. An HDR file infers `Rgba16Float`, where `Rgba32Float` never uploaded.
- Raw bytes must match their region exactly. `create_texture` with a size, the `create_storage_texture` seed and `Mipmap.build` refused only short data, and a longer buffer uploaded its first rows. `Texture.write` and `write_region` without a stride take tight or 256-byte padded rows, and any other length returns `TextureError::InvalidInput`, which Python raises as `InvalidInputError`. `Mipmap.build` refuses a size with a depth past 1.
- A partial write into a texture with a mip chain returns `TextureError::InvalidInput`, since it would leave the smaller levels stale. Create a texture you update in parts with `mipmaps: false`.
- A size with a path, a URL, a KTX2 file or a `Mipmap` returns `TextureError::InvalidInput` from `create_texture` and `Mipmap.build`, before any file read or fetch. A path dropped the size, and a URL uploaded its encoded bytes as pixels.
- A JavaScript `writeRegion` object and a Python `write_region` dict need their `width` and `height`, or their `maxX` and `maxY`, each max past its min, and one form at a time. Reversed bounds wrote to the edge of the texture, and misspelled keys wrote the whole texture.
- A JavaScript `ImageData` or canvas takes the size it has and an 8-bit RGBA or BGRA format, and a NumPy array takes its own size. A different `size`, or another format, throws or raises `InvalidInputError`.

## 0.12.6: Binding and renderer fixes

A patch release that fixes crashes, memory leaks, and wrong output, and closes parity gaps between the bindings. Every documented example now compiles and runs in Rust, JavaScript, Python, Swift, and Kotlin. Changes that can break code come last.

### Renderer and meshes

- `Pass.set_viewport` takes effect. A pass draws only inside its region, the part past the target is cropped, and the clear color still fills the whole target. The renderer ignored the viewport.
- Each pass in one `render` call keeps its own uniforms instead of drawing with the last pass's values.
- Shaders that share a source get one render pipeline per vertex layout, and compute passes reuse theirs. A mesh with another layout drew wrong vertices, and every frame leaked layout memory and compiled and kept new compute pipelines.
- Passes, shaders, and their storage buffers are freed when the last handle drops. A simulation of a million particles leaked about 30 MB per rebuild.
- Storage buffers upload only when created, resized, or changed by `set`. At eight million particles, each frame copied about 256 MB.
- `get_image` and `read_storage` drop their pooled buffer when they time out or fail. Later readbacks no longer fail with `BufferStillMapped`.
- With only compute passes, a `render` acquires no surface frame and keeps running while a window is minimized.
- A shader with two `@vertex` or `@fragment` functions draws with the first of each. Render pipelines took the last.
- When a mesh's instance list shrinks, the mesh draws only the instances it holds, not the old count.
- Vertices or instances that disagree on an attribute's format or length make the mesh fail to attach, where later attributes shifted. A value added after the attach fails the render with `MeshError`.
- `Scene.load` reads glTF files with application attributes such as `_ID`.

### Shaders and uniforms

- `Shader.set` follows one set of rules in every language. Integers must be whole and in range, vectors and matrices take exact counts, and errors read like `resolution: expected vec2<f32>, got 3 number(s)`.
- Integer uniforms work on the web, and `mat2x2` in JavaScript, Swift, and Kotlin. Indexed keys get the type check, Python raises on a list of the wrong length, and `Shader.get` returns current values, where struct roots returned defaults.
- `mat3x3<f32>` values fill 48 bytes with WGSL's column padding. Packed into 36, the last two columns shifted.
- On the web, `Shader.set` binds only a real `Texture` and throws for any other object rather than reading its memory as a texture.
- Storage buffers hold `atomic<u32>` and `atomic<i32>`, and another shader can read them as `array<u32, N>`.
- `Shader.set_registry` works on native builds, with the embedded library as fallback. Full URLs into the default registry use it too, which ends the network errors on Swift and Kotlin.
- The registry's `sdf2d/heart` returns Inigo Quilez's exact signed distance, and outlines and glows trace the shape. `pattern/hexgrid` draws hexagons instead of tall, overlapping cells.

### Textures

- Sampler options take effect at creation and after `set_sampler_options`, where every `sampler` bound the defaults. A `sampler` takes its group's texture options, and PBR's shared sampler follows the base color map. `sampler_comparison` defaults to `LessEqual`.
- `Texture.write` and `write_region` take tightly packed rows: a 2×2 RGBA write needs 16 bytes, not the 264 it asked for. Padded rows still upload, and `bytes_per_row` only has to cover the row.
- `create_texture` honors `TextureOptions.usage`: `TextureUsage::COPY_SRC` makes a texture readable. `read_texture` failed on every texture `create_texture` made.
- Python texture methods read NumPy arrays in row-major order. Fortran-ordered arrays uploaded scrambled pixels, and flipped or sliced views raised `ndarray must be contiguous`.
- Swift `createTexture(pixels, size)` builds a full mip chain and throws for a size of other than 2 or 3 numbers. The old call decoded the pixels as an image.

### Binding fixes

- Swift and Kotlin `pass.setDepthTarget(texture)` hand Rust the `Texture` itself. They passed its handle as a `TextureTarget`, which is undefined behavior.
- Android `renderer.createTarget(surface)` works again after failing with `Failed to create surface` on every call since 0.12.3. The target lives on the calling renderer, and `render` no longer fails with `NoContext`.
- iOS `createTarget` throws `Init` for a null `CAMetalLayer` pointer instead of reading through it.
- `Model(mesh, material)` and `model.material()` share the `Material` in every binding. A rebuilt copy reset its alpha mode and drew `Mask` and `Blend` materials wrong.
- `Pass.require` links each dependency once on Swift and Kotlin, and for a list on the web. The docs example failed on mobile with `DuplicateDependency: color`. Python `render` runs the passes a `Pass` requires.
- On the web, `scene.setPasses` copies its handles, and later calls stop throwing `Attempt to use a moved value`. `render` records a list as one frame instead of letting the last item replace the others, and `load` and `WindowTarget.getImage()` no longer throw.
- Python `Renderer.create_target` keeps one target per renderer, where it shared a canvas's target across devices.
- Swift `writeRegion` takes 2, 4, or 6 numbers and throws for other arrays, which it ignored. `Vertex.set` and `Instance.set` throw for a list of other than 1 to 4 numbers, and `TextureRegion.from` traps with a message on an array it can't read. All three misread such input.

### Parity across languages

- Every binding uses Rust's type names, such as `WindowTarget` and `TextureRegion`. `CanvasTarget`, `RenderCanvasTarget`, `TextureRegionMobile`, `MobileWindowTarget`, and `MobileTextureTarget` remain aliases.
- `Pass.add` and the `Scene` builders return the receiver, and `scene.add(model).add(camera)` chains. `Pass.require` takes a `Scene`. `ScreenRegion(origin, size)`, `[x, y, w, h]` viewports, and `[r, g, b]` clear colors work in every binding.
- `Shader.set` takes each platform's number types, matrices flat or as columns, raw bytes for storage and push-constant roots, and bools as 1 or 0. Kotlin takes a `List<UInt>`, as Swift takes `[UInt32]`. Rust's `set("scale", 2)` fills an `f32`, and `Shader::get` reads matrices and bytes.
- Swift and Kotlin `createTexture` takes bytes alone or with a format, a `[w, h]` size, or options, and the `TextureUsage` flags exist everywhere. Rust accepts path and URL strings, the web reads every option, and JavaScript and Python load KTX2.
- Python has one readable `Size` class, and `texture.size.width` no longer raises `AttributeError`. It also builds `SamplerOptions` and `CompareFunction`, and its value types compare by value.
- A number or list of numbers on a `Vertex` or `Instance` takes the type the shader declares at its location when the mesh attaches, as uniforms do. `vertex.set("id", 7)` fills an `id: u32` input in every language, and a value that doesn't fit fails the attach with an error like `Mesh attribute id: expected u32, got -1 at vertex 2`.
- `Vertex.UV0` and the other constants reach all five languages, along with `(position, uv, color)` vertices. Python `render` takes a `Mesh`, and `Scene.load` reads `.glb` bytes and can skip cameras or lights.
- `Material` texture setters accept any `createTexture` source, Kotlin scene types take `Double`, and the TypeScript typings allow plain arrays.

### Docs and generated examples

- The iOS and Android healthchecks run each documented example as a test named after its `docs/api` page instead of only compiling it.
- Uncaptured GPU errors go to the file that `FC_GPU_ERROR_LOG` names and fail the iOS, Android, and Python healthchecks.
- Generated examples match their Rust source, and the build removes those of deleted pages. Kotlin gets `createTexture(pixels, listOf(w, h))`. Its examples sent raw pixels to the image decoder and failed.
- Method links land on their heading rather than a 404, and each method's anchor is its name, as in `/api/scene/scene#remove_pass`. `Instance` and `Mipmap.level` have pages now, and four missing method pages reach the site.
- The regenerated Python stub gains `Scene`, `Mipmap`, and the other types it lacked.

### Website and examples

- The website opens on a playground, where the shader rebuilds as you edit WGSL and JavaScript. Controls come from its uniforms, and the same program shows in Rust, Python, Swift, and Kotlin. `/demos` redirects there.
- Among its demos are the shader artworks and two compute demos with a million particles each, which need WebGPU.
- The `particles` example grows particle life on a grid. Four species follow or flee each other's density, and the mouse, R, C, and the arrow keys steer it. The three broken particle examples are gone.

### Packaging and CI

- Each release tag pins its own xcframework, where `from: "0.12.5"` got 0.12.4's binary. Move Swift dependencies to 0.12.6.
- SwiftPM compiles a regenerated wrapper. The committed one predated 0.12.0 and lacked `Scene`, `Model`, and the other 0.12 types.
- FragmentColor builds from read-only sources. docs.rs documents it again, after no docs for 0.10.7 through 0.12.5.
- The Python wheel requires `numpy>=1.16`, without which `TextureTarget.get_image` panicked. The npm package drops the test harness's `WasmBindgenTestContext` and `__wbgtest_*` exports.

### Breaking and behavior changes

- JavaScript methods throw `Error` objects, where most threw bare strings. Read `e.message` instead of comparing the caught value to a string.
- On the web, invalid source makes `new Shader(source)` throw and `Shader.fetch` reject. Both returned the red fallback that `Shader.default()` still gives. Paths and slugs need `Shader.fetch`, and a live editor can keep its last good `Shader`.
- Where Rust takes a `u32` or `u64`, such as indices, counts, sizes, and regions, Swift takes only `UInt32` or `UInt64` and Kotlin only `UInt` or `ULong`. An `Int` variable fails to compile; check its sign and convert with `UInt32(n)` or `n.toUInt()`.
- Swift literals compile as written, as in `createTextureTarget([64, 64])`, and Kotlin writes `64u` and `listOf(64u, 64u)`. The removed `Int` overloads trapped on or wrapped negatives. Byte lists keep their `Int` forms and throw outside 0 to 255.
- Python `get_image` returns a `uint8` array shaped `(height, width, ...)`, layers first for array textures. `TextureTarget` returned the transpose: swap `[x, y]` indices. `Texture` returned flat bytes, which `bytes(image)` still gives.
- Renderer and texture failures in Python raise `FragmentColorError` or a subclass. They used to raise a plain `Exception`, and `read_storage` a `RuntimeError`. Catch `FragmentColorError`. `Scene.load` raises `SceneLoadError`, also a `RuntimeError`, and light setters `LightError`, also a `ValueError`.
- For an argument of the wrong type, Python `Shader`, texture, mesh, and `Pass.require` calls raise `TypeError`, where most raised `FragmentColorError`. Handlers there need `TypeError`. `require` rejects a `str`, and `Shader.set` refuses the `TextureMeta` that `Shader.get` returns.
- `Vertex.set` and `Instance.set` in Python raise `FragmentColorError` for a list of other than 1 to 4 numbers. A position passed as an instance raises `TypeError` and no longer adds an empty instance.
- A Python window target's `depth` is `None` (it was 1), and `create_texture` reads `http(s)` strings as URLs.
- JavaScript and Python sizes take 2 or 3 numbers, regions 2, 4, or 6, and bytes 0 to 255, each finite and non-negative. Anything else fails, where JavaScript clamped or zeroed it.
- Indices, counts, and `setComputeDispatch` sizes in JavaScript take whole numbers instead of wrapping -1 to 4294967295. On the web, `Pass.require`, `createTexture`, `Vertex.set`, and `Instance.set` throw for wrong types. They ignored such values or read them as 0.
- A region built from a size covers exactly that size. The extra pixel per side that `ScreenRegion::from([w, h])` added failed `write_region`. Swift `setViewport([x, y, w, h])` reads an origin and size, not two corners.
- `create_target` returns `InitializationError` when the adapter can't present, as on a second WebGL2 canvas, which needs its own `Renderer`. It used to succeed and only log.
- Swift and Kotlin report a GPU that fails to start as `FragmentColorError.Init` from every renderer method, where several said `Render`. Kotlin argument errors throw `FragmentColorException` in place of `IllegalArgumentException`.
- `UniformValue` replaces `UniformData` in the generated `set(key:value:)`; documented `shader.set` calls are unchanged. Swift `addVertex` and `addVertices` don't throw, and their calls drop the `try`, while `Vertex.set` and `Instance.set` throw now and need a `try`.
- `Pass.get_input` and `PassInput` leave the API in every binding, and Swift and Kotlin lose `MobilePassInput`. A pass's settings are write-only: `load_previous`, `set_clear_color`, `set_viewport`, and `set_compute_dispatch`.
- The bindings retire `VertexValue`. Pass your language's numbers to `Vertex.set` and `Instance.set`. Rust's `set` takes `impl Into<VertexData>`, a transport type with no page of its own: an `f32` or `u32` keeps its type and must match the input, and an unsuffixed literal, `f64`, or `i32` takes the shader's type.
- A vertex or instance attribute set from a plain number takes the type the shader declares at its location. The bindings sent every number as an `f32`, and Rust read an unsuffixed integer as an `i32`. JavaScript `Int32Array` and `Uint32Array` values count as plain numbers too. The attach checks every vertex and instance rather than only the first. Once a shader types an attribute, that type holds for the mesh, and a shader that declares another type fails to attach.
- `PassError::NoCompatibleShader` carries the reason the pass's last shader gave, and a pass with no shader returns `PassError::NoShader`.
- Rust `Vertex::set` and `Instance::set` are `#[must_use]`: use their result. Swift warns on an unused result from `Instance.set` and from generated builders such as `noDefaults()`.
- Kotlin `Shader.setRegistry` returns nothing instead of a throwaway `Shader`. Rebuild libraries compiled against 0.12.5.
- Swift deprecates `Shader.new([...])` for `Shader([...])`, `setRegistry(baseUrl:)` for `setRegistry(_:)`, and the tuple forms of `createTexture` and `createStorageTexture`. Kotlin deprecates `ShaderFetch` for `Shader.fetch`, and Python `usage_bits=` for `usage=`.
- Samplers with no texture in their group filter linearly and no longer pick the nearest texel. glTF textures follow their glTF sampler. UVs outside 0 to 1 tile unless the file's sampler clamps.
- `pattern/hexgrid` returns the edge distance and an integer id, `x` in half cells and `y` the row. The cell center it returned split hashed cells into stripes along the axes. Its page derives the center.
- The Python stub drops plumbing such as `passes`, which the bindings lose in 0.13.
- Rust items that were public by accident leave the API: the built-in shader sources, the wgpu device setup helpers, `RenderContext`, the buffer and texture pools, `TextureData::DynamicImage`, and the glam and wgpu conversions of `ScreenRegion` and `UniformData`. Pass `v.to_array()` where code passed a glam vector.

## 0.12.5: WebGL2 fixes

A patch release that makes the WebGL2 fallback work. There are no API changes.

Shaders with uniform bindings whose size is not a multiple of 16 bytes, such as a bare `f32`, a `vec2<f32>`, or most structs, failed to build a pipeline on OpenGL and WebGL2 because wgpu requires 16-byte aligned bindings on those backends. FragmentColor now pads them for you: on a backend that needs it, each shader compiles a padded module in which uniform structs get a padded copy and bare uniforms a wrapper, with every use rewritten. Uniform names, keys, and offsets stay the same, so no shader source has to change.

`TextureTarget::get_image` and `Renderer::read_storage` hung on WebGL2, where the buffer map callback only fires when the device is polled. The web readback path now polls and yields to the event loop until the map resolves.

GPU validation errors now reach the browser console. The error handler printed through stdout, which WebAssembly discards; it logs through the console instead. The web healthcheck gained a second pass that runs with `navigator.gpu` removed so the WebGL2 path stays covered.

The repository root is tidier for first visitors. The build, run, and check scripts live in `tools/` behind a single `./fc` dispatcher (`./fc` lists them, `./fc test` runs one), the `uniffi-bindgen` helper moved into `src/bin`, the npm and PyPI READMEs sit beside their packages under `platforms/`, and three stale files are gone. CI and the documentation call the new paths.

## 0.12.4: Resize and grain fixes

A patch release with two fixes. There are no API changes.

`TextureTarget::resize` now keeps the renderer's texture registry in sync. Resizing an offscreen target used to swap its internal texture while the registered id kept pointing at the old one, so every `Pass` color attachment and shader binding that referenced the target kept drawing into and sampling the stale texture. Passes and bindings now follow the resize automatically, and recreating targets as a workaround is no longer needed.

The `postfx/film_grain` catalog function now produces per-pixel grain when given the normalized fragment coordinate. Its hash used to collapse into smooth diagonal banding for inputs in [0, 1] because the internal multiply never left the first fract period. The function scales its input internally, so call sites that pass `in.uv` render correct grain. The catalog preview and docs page are updated to match.

## 0.12.3: wgpu 30

A maintenance release that moves the graphics stack to `wgpu` 30 and `naga` 30. The public API does not change, so it is a drop-in upgrade on every binding.

Inside, the renderer follows the wgpu 30 surface API. A frame now presents through `Queue::present` rather than `SurfaceTexture::present`. The swapchain configuration gains a `SurfaceColorSpace`, kept at `Auto` so colors match earlier releases. Vertex buffer layouts fill optional slots, and buffer readback returns a `Result` instead of panicking when a map fails.

The language bindings move forward as well: `pyo3` and `numpy` to 0.29, and `uniffi` to 0.32. A `cargo update` refreshes every other dependency to its latest compatible version.

## 0.12.2: Composable Scene

`Scene` now exposes its render passes the way `Pass` and `Renderer` already do. A scene owns one ordered list of passes that you can read, append, remove, reorder, and replace, so a scene loaded from glTF composes on top of another pass in a frame instead of clearing it. `Scene::load(...)` is unchanged and still returns a renderable scene with sensible defaults.

What's new:

- Read and edit the pass graph with `add_pass`, `remove_pass`, `get_pass`, `find_pass`, `list_passes`, and `set_passes`.
- Add a `Model`, `Camera`, or `Light` to a chosen pass by index or name with `add_to`.
- Decide whether the stock Camera and Light inject at first render with `no_defaults`, `no_default_camera`, `no_default_light`, `set_default_camera`, and `set_default_light`.
- Read a pass's label with `Pass::name`.

One behavior change: passes render in list order, so a pass added after the geometry now renders after it. A pass added before it (the common backdrop case) is unaffected.

Every method ships on Rust, Python, JavaScript, Swift, and Kotlin.

## 0.12.1: Publishing fix

A patch release that gets 0.12.0 onto crates.io. The 0.12.0 upload failed because `build.rs` generated the `Scene::load` example fixtures into the source tree, and `cargo publish` rebuilds the crate in a clean directory and rejects a build that writes there. The fixtures (`path/to/model.glb`, `path/to/model.gltf`, and the web healthcheck copy) now ship as committed static assets, and the build no longer writes them. There are no API changes.

## 0.12.0: 3D scenes, glTF loading and PBR materials

A large release that adds a 3D layer on top of the core: a `Scene` container, a glTF 2.0 loader, physically based materials, cameras, and lights. Every new type and method ships on Rust, Python, JavaScript, Swift, and Kotlin.

### Scenes

- `Scene` is the top-level container for a 3D frame. It holds passes and implements `Renderable`, so `renderer.render(&scene, &target)` draws it in one call. `Scene::new()` is synchronous and allocates nothing until the first object arrives.
- `Scene::add` and `Pass::add` take a `Model`, a `Camera`, or a `Light`, or any type that implements the `SceneObject` trait. A model fails to attach when its mesh does not match its material's shader, so both return a `Result`.
- `Scene::add_pass` adds a pass that renders before the scene's own geometry, such as a shadow map or a backdrop. `Scene::models`, `cameras`, and `lights` list what the scene holds.
- A scene with models but no camera or light gets defaults on its first render: a perspective camera at `[0, 0, 5]` looking at the origin, and a directional light. Adding your own camera or light turns its default off.
- `Scene::ambient([r, g, b])` sets the ambient light for every shader in the scene, before or after models arrive.
- `Model` pairs a `Mesh` with a `Material` and a 4×4 transform. `translate` moves it in world space, `rotate` and `scale` work around its own origin, `set_transform` and `transform` write and read the whole matrix, and `set_visible` hides it. Models that share a mesh and a material draw in one instanced call.
- `Camera::perspective` and `Camera::orthographic` build cameras for wgpu's `[0, 1]` depth range. `look_at` aims one, `set_aspect` follows a resize, and `view_proj` and `position` read it back. A camera updates every shader it is attached to whenever it changes.
- `Light::directional`, `Light::point`, and `Light::spot` follow glTF's `KHR_lights_punctual` model. Each has getters and setters for color, intensity, position, direction, range, and cone angles, and changes reach every shader that uses the light. The PBR shader takes up to 32 lights; more log a warning and are dropped.

### glTF 2.0 loading

- `Scene::load(source)` reads a `.gltf` file with external buffers and images, or a `.glb` container, from a path or from bytes, and returns a scene that renders with the defaults above. Python and JavaScript accept a path or bytes; Swift and Kotlin take a path.
- The loader covers mesh primitives with optional normals, UVs, and indices; every metallic-roughness material field and texture slot; node transforms in matrix or TRS form; perspective and orthographic cameras; and punctual lights.
- Files from real authoring tools load correctly too. Primitives without normals get smooth normals computed from their faces. `COLOR_0`, `TEXCOORD_1`, and `TANGENT` are read, with a fixed tangent when a file has none. Each texture keeps its sampler's wrap modes and filters. `KHR_texture_transform` on the base color map becomes the material's UV transform.

### PBR materials

- `Material::pbr()` returns FragmentColor's physically based shader (Cook-Torrance with GGX, Smith, and Schlick terms, and Lambert diffuse) with glTF 2.0 metallic-roughness defaults. `Material::custom(shader)` applies the same setters to your own shader wherever its uniforms match.
- Setters cover `base_color`, `metallic`, `roughness`, `normal_scale`, `occlusion_strength`, `emissive`, and `alpha_cutoff`, the five texture slots (base color, metallic-roughness, normal, occlusion, emissive), and `uv_transform(offset, scale, rotation)`.
- Texture setters accept anything `create_texture` accepts: a `Texture`, bytes, a path, a URL, or an image. Sources that are not textures yet upload on the first render on native, and `Renderer::load(&renderable)` uploads them ahead of time. On the web, `render` cannot wait, so call `await renderer.load(...)` first. Base color and emissive maps default to sRGB, the other maps to linear.
- Unset texture slots fall back to neutral 1×1 textures, so a material renders before any of its textures exist.
- Normal maps use a full tangent-space transform, vertex colors tint the albedo, and the shader carries a second UV set.
- The shader's building blocks, `mesh/transform` and `material/pbr`, are registry snippets that compose into your own shaders, behind the `shaders-mesh` and `shaders-material` features.

### Transparency and culling

- `Material::alpha_mode` takes `AlphaMode::Opaque`, `AlphaMode::Mask` (fragments below `alpha_cutoff` are discarded), or `AlphaMode::Blend` (alpha blending with depth writes off). `Material::double_sided` turns back-face culling off; materials are single-sided by default.
- Blended models in a pass draw back to front from the camera, whatever order they were added in. Opaque and masked models keep the instanced path.
- Behavior change for shaders rendered without a material: blending is off, where every pipeline used premultiplied alpha blending before, and both faces render. To blend again, wrap the shader in `Material::custom(shader).alpha_mode(AlphaMode::Blend)`.

### Meshes and vertices

- Attribute name constants: `Vertex::POSITION`, `NORMAL`, `TANGENT`, `UV0`, `UV1`, `COLOR0`, and `COLOR1`. `vertex.set(Vertex::UV0, ...)` is the same call as `vertex.set("uv0", ...)`.
- `Vertex::pbr(position)` seeds the five PBR attributes (`NORMAL`, `UV0`, `COLOR0`, `UV1`, `TANGENT`) with neutral defaults. Chain `set` to override the ones you have.
- `Mesh::set_indices` uses your own index buffer and skips vertex deduplication. `Mesh::clear_indices` returns to automatic indices.
- Shader inputs match mesh attributes by name first and by location second. Matching by location first sent attributes to the wrong inputs when vertex and instance locations overlapped.

### Textures

- `TextureMipChain` becomes `Mipmap`. `prepare` becomes `build`, `base_size` becomes `size`, and `level_count` becomes `count`; `format` and `levels` keep their names.
- The API docs gain a `texture/` group that holds `Texture` and `Mipmap`. `core/` keeps `Renderer`, `Shader`, and `Pass`.

## 0.11.1: Embedded shader registry by default

A patch release that fixes the Linux PyPI publish: 0.11.0 reached crates.io, npm, the Swift xcframework, and the Android AAR, but its PyPI wheels did not. The fix settles a design question too: a graphics library should not pull a TLS stack into every consumer's dependency tree.

### Defaults

- **The whole public shader library (about 86 KB of WGSL across 233 files) is embedded by default** on every native build. `Shader::new("sdf2d/circle")` resolves from the binary, with no network. Slim it down with `default-features = false` if size matters.
- **Registry URLs resolve from the embedded library.** A URL of the form `<registry-base>/<category>/<name>.wgsl` (default base `https://fragmentcolor.org/shaders/`) counts as a registry URL and resolves locally on native. So `Shader::new("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")` and `Shader::new("sdf2d/circle")` give the same result, and the longer URL form keeps working in docs and quickstart examples on every platform.
- **The web build (`build_web`) leaves the embedded library out** through `--no-default-features`. Slugs and registry URLs resolve with the browser's `fetch()` against the live registry, which keeps the wasm bundle small.
- **`network` is a new Cargo feature, off by default on every native target.** With it on, `ureq` builds with `native-tls`, so URLs outside the registry (`https://example.com/foo.wgsl`, and so on) and `Renderer::create_texture(url)` fetch over the system's TLS stack (OpenSSL on Linux, Schannel on Windows, Secure Transport on macOS). With it off, those calls return `NetworkError::feature_disabled()`, whose message says to rebuild with `--features network`.
- **The API does not change.** `Shader::fetch` and `Renderer::create_texture` accept the same inputs on every binding. Slugs, registry URLs, file paths, and raw source work everywhere; only URLs outside the registry depend on how the binary was compiled.

### Internal

- `crate::net::NetworkError` replaces direct `ureq::Error` references in `FragmentColorError::NetworkRequest`, `RendererError::NetworkRequestError`, and `ShaderError::RequestError`. With the feature on, `From<ureq::Error>` keeps `?` working at existing call sites.
- `ureq` is an optional dependency behind the `network` feature. Without the feature, `ring`, `rustls`, and `openssl-sys` leave the default dependency tree, and PyPI wheels build on every Linux architecture.

### CI

- `.github/workflows/publish_py.yml` is regenerated with maturin 1.13.1. It picks up newer action versions (`actions/checkout@v6`, `setup-python@v6`, `upload-artifact@v6`, `download-artifact@v7`, `attest-build-provenance@v3`), drops the Node.js 20 deprecation warnings, publishes with `uv publish`, builds wheels on every pull request so cross-compile breakage shows before a release, and adds a Windows ARM64 target.

### Metadata

- The PyPI development status moves from `2 - Pre-Alpha` to `3 - Alpha`. Five language bindings ship, and the API is settling, with changes still expected before 1.0.

### Migration

Most call sites need no changes. Registry URLs (`https://fragmentcolor.org/shaders/<category>/<name>.wgsl`) resolve from the embedded library on native, with no network and no code changes.

For URLs outside the registry on native:

- **If a registry slug or registry URL works for you,** use it. It works offline on every platform.
- **If you need an arbitrary URL on native,** build with `--features network` (`cargo add fragmentcolor --features network`, or rebuild the Python wheel with `maturin build --features network`).
- **The web binding is unchanged.** `Shader.fetch(url)` keeps working through `fetch()` in the browser for any URL.

## 0.11.0: Swift and Kotlin

The first release with Swift and Kotlin bindings, built with [uniffi](https://github.com/mozilla/uniffi-rs). Struct names match the Rust core (`Renderer`, `Shader`, `Pass`, `Size`, `WindowTarget`, `TextureTarget`, `Texture`, `SamplerOptions`, `CompareFunction`), so the API reads the same on every platform. The release also composes shaders from the registry, loads KTX2 textures, moves texture work off the main thread, and upgrades to wgpu 29.

### Swift and Kotlin

- On iOS, `Renderer.createTarget(layer:)` builds a `WindowTarget` from a `CAMetalLayer`, through `Renderer::create_target_ios(metal_layer_ptr: u64)`.
- On Android, `Renderer.createTarget(surface:)` builds one from an `android.view.Surface`, through a JNI entry point, since uniffi cannot pass a `JNIEnv*`.
- `createTextureTarget`, `render`, `Shader(source:)`, and `Texture.setSamplerOptions(...)` work on both, with `SamplerOptions` and `CompareFunction` exported as uniffi types. `render` takes a shader, a pass, a mesh, or a list of passes, and a window or texture target.
- `createExternalTexture(sourcePtr)` takes a `CVPixelBuffer` or `SurfaceTexture` pointer. Like every platform's implementation, it returns "not implemented yet".

### Packaging and CI

- A root `Package.swift` lets Swift Package Manager depend on the repository URL (`from: "0.11.0"`). It downloads `fragmentcolor.xcframework.zip` from the GitHub Release and checks its SHA-256.
- The Kotlin AAR ships as a GitHub Release asset.
- `build_ios` builds the xcframework for `aarch64-apple-ios` and the simulator, and `build_android` builds all four Android ABIs with `cargo-ndk`.
- New workflows: `healthcheck_ios.yml` (Xcode on macos-14 with an iPhone simulator), `healthcheck_android.yml` (an Android emulator on ubuntu-latest with KVM), `publish_swift.yml`, and `publish_android.yml`. `post_publish_update.yml` pins `Package.swift` to the released checksum.
- The build adds `uniffi` 0.29 on non-wasm targets, with a `uniffi-bindgen` binary and a `uniffi.toml`. iOS adds `objc2-foundation` and `objc2-quartz-core`, and Android adds `jni`, `jni_fn`, `ndk-sys`, and `raw-window-handle`.

### Generated examples and docs

- The build transpiles every `docs/api` Rust example to Swift and Kotlin, and the iOS and Android healthchecks compile them: `platforms/swift/healthcheck/GeneratedExamples.swift` and `platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/GeneratedExamples.kt`. They compile the examples without running them.
- The transpilers translate more Rust idioms for every target language: method chains split over several lines, numeric suffixes (`64u32` becomes `64`), unary dereferences, array-repeat literals such as `[0u8; 256]`, bare raw-string assignments, single-quoted strings in Swift and Kotlin, and Kotlin `arrayOf(...)` literals. The generated Swift parses with no errors.
- The `Pass::add_depth_target` page explains that attaching a depth target turns on depth testing and depth writes, and that a pass without one draws in submission order.

### Shader composition

- `Shader::new` accepts a list. Each part is WGSL or GLSL source, a registry slug such as `"sdf2d/circle"`, an `https://` URL, or a local path. Parts resolve, deduplicate by content, and join in order before naga validates them. Calls with one string keep working.
- `Shader::set_registry(base_url)` changes where slugs resolve. The default is `https://fragmentcolor.org/shaders/`.
- GLSL works only as a single part.
- JavaScript: `new Shader(input)` takes a string or a list, and `await Shader.fetch(input)` resolves URLs and slugs. Python: `Shader(input)` takes a string or a list. Swift and Kotlin: `Shader.new(source)` and `Shader.compose(parts)`.
- Behavior change: a single-line string shaped like `category/name` resolves as a registry slug instead of parsing as WGSL.

### Textures

- `Renderer::create_texture(input)` and `Renderer::create_storage_texture(input)` are the only ways in, with an optional options argument in JavaScript and Python. `TextureInput` pairs a `TextureData` source with `TextureOptions` (size, format, sampler, mipmaps, usage). Swift and Kotlin pass a `TextureInputMobile` and a `TextureOptions` record. A storage texture can start from tightly packed bytes, with no row padding.
- On native targets, decoding, mipmap generation, and uploads run on a background worker, so `create_texture` no longer blocks the calling thread. The web build runs them inline.
- `TextureMipChain::prepare(input)` builds a mip chain on the CPU ahead of time, on every binding, and `create_texture` takes the chain.
- Source images get a full mipmap chain, and the default linear sampler filters between mip levels, which stops the shimmer on detailed textures seen from afar. `TextureOptions.mipmaps` (default `true`) skips it.
- `TextureOptions.format` applies to every input, so an RGBA PNG can load as linear `Rgba8Unorm` for a normal map.
- Images decode into the layout of their target format, so 16-bit PNGs keep their precision, and a 16-bit PNG loaded by path no longer has garbled rows. `TextureFormat` gains `R16Unorm`, `Rg16Unorm`, and `Rgba16Float`. JavaScript code that passes format numbers should re-read them, since the numbering shifted.
- `Rgba16Float` is filterable and storage-writable in core WebGPU, which suits simulations that lose precision in 8-bit storage.
- KTX2 containers load through `TextureData::Ktx2Bytes`, `Ktx2Path`, and `Ktx2Url`, parsed in pure Rust. The file's format and mip chain are used as they are: RGBA8 and BGRA8 (UNORM and sRGB), R8, Rg8, R16, Rg16, and Rgba16 UNORM, RGBA16F, BC1 to BC7, ETC2, and ASTC 4×4 and 8×8. The device requests BC, ETC2, and ASTC support where the adapter has it, and an unsupported format fails with a clear error.
- The device also requests `TEXTURE_FORMAT_16BIT_NORM` and `FLOAT32_FILTERABLE` where available. Without them, a 16-bit norm texture fails at creation with `TextureError::UnsupportedFormatForUsage`, where it used to fail on every frame.
- GPU validation errors from bind group and view creation return as `RendererError::ValidationError { label, message }`.

### Compute

- `Renderer::read_texture(id)` and `Texture::get_image()` return a texture's first mip level as tightly packed bytes in its own format, so a storage texture no longer needs a trip through a `TextureTarget` to be inspected.
- Textures that a shader reads only through `textureLoad` bind as non-filterable, so `Rgba32Float` works as a sampled source without `FLOAT32_FILTERABLE`.

### Fixed

- Sampled textures and samplers are visible to compute shaders. Compute pipelines that sampled a regular texture used to fail to build.
- `texture_storage_2d<..., read_write>` binds as read-write. It was bound read-only, so every `textureStore` failed validation.
- `Texture::get_image` no longer deadlocks on native.
- On Apple GPUs, two compute passes in a row run as separate submissions, so the second sees the storage-texture writes of the first. It used to read zeros. No opt-in is needed.

### Breaking changes

- `Frame` is removed. `Renderer::render` takes a `Pass` or any list of passes, and `Pass::require()` builds dependency graphs. Replace `frame.add_pass(&a); frame.add_pass(&b); renderer.render(&frame, &target);` with `renderer.render(&vec![a, b], &target);`, or `renderer.render([a, b], target)` in Python and JavaScript.
- `Renderer::create_external_texture_from_html_video` becomes `Renderer::create_external_texture`, now on every binding with a source type per platform. Every implementation is a stub, so its page stays off the website.
- Method names follow one rule: one verb, or at most three words. `Pass::add_mesh_to_shader` is gone; call `shader.add_mesh(mesh)`. `App::on_event_kind`, `on_window_event_kind`, and `on_device_event_kind` become `on_event`, `on_window_event`, and `on_device_event`, and the variants without a kind are gone.
- `Target` gains an async `get_image()`, which replaces `TextureTarget::get_image_async`. `WindowTarget::get_image` returns an empty vector.
- `create_texture_with`, `create_texture_with_size`, and `create_texture_with_format` fold into `create_texture`, which takes the options inside its input.

### Dependencies

- `wgpu` and `naga` move from 27.0.1 to 29.0.1, with no change to FragmentColor's API. `fragmentcolor::SurfaceError` replaces `wgpu::SurfaceError`, which wgpu removed, with the same variants. WGSL `var<push_constant>` keeps working: FragmentColor rewrites it to `var<immediate>`, the only form naga's WGSL front end accepts now.

### Known issues

- On Apple GPUs, a render pass that samples a texture written by a compute pass in the same command buffer can read zeros. Render the two passes with separate `render` calls.
- Some generated Swift and Kotlin examples fail to type-check: those that create a headless window, call `Shader::default()` or `Shader::from_mesh()` (not exported to Swift and Kotlin), or pass `[width, height]` where the uniffi signature takes two integers.
- Generated examples for deleted docs pages stay on disk under `platforms/`.

## 0.10.11: Texture writes and HTML video textures

### Added

- `Texture::id`, `Texture::write`, and `Texture::write_with`.
- `Renderer::update_texture`, `Renderer::update_texture_with`, and `Renderer::unregister_texture`.
- `TextureWriteOptions` sets the upload origin, size, bytes per row, and rows per image.
- Web: `Renderer::create_external_texture_from_html_video` samples an HTML video through `texture_external`.

### Python and RenderCanvas

- Works with RenderCanvas versions that dropped string-based `get_context("fragmentcolor")` integrations. `Renderer::create_target` reads the screen's present info directly, creates the surface from it, and reuses the cached `RenderCanvasTarget`.

### Docs and generation

- Docs and generated JavaScript and Python examples for the texture-write APIs.
- Override snippets per language where the generic Rust conversion falls short.

## 0.10.10: Web glue guard and ArrayBuffer fixes

### Web

- `build_web` patches the generated wasm-bindgen glue: when the `Uint8Array(ArrayBuffer)` constructor meets a detached buffer, it falls back to `new Uint8Array(wasm.memory.buffer)`, which avoids crashes in long sessions.
- `TextureInput` from JavaScript treats an `ArrayBuffer` with `byte_length() == 0` as detached and empty, and returns no bytes instead of throwing.
- `ImageData` and canvas reads copy through `ImageData.data().0`, which replaces an incorrect `copy_to()` on clamped data.

### Website and tooling

- The hero shader component is simpler, with its per-frame instrumentation removed.
- `run_docs`, `astro.config.mjs`, and the lockfiles for the site and examples are updated.

## 0.10.9: Stable class branding in minified JavaScript

### Fixed

- Each type carries a stable kind brand, so minified JavaScript builds still recognize FragmentColor objects. Types register with `crate::impl_fc_kind!(TypeName, "TypeName")`, and the crate exports `fc_kind` and `FcKind`.

## 0.10.8: Uniforms that never block, typed errors, and a web gallery

### API changes

- `RendererError` gains `MsaaViewMissing` and `DepthSampleCountMismatch`, and `InitializationError` gains `AdapterNotSet`.
- `ShaderError` gains `Busy`. `set()` no longer blocks: it queues updates, and the last one wins. Read methods may return `Busy` briefly under contention.

### Internals

- Uniform updates queue without blocking, and the renderer applies them before binding.
- A `Pass` takes its kind (render or compute) from its first shader.
- The web build grows linear memory by 64 MiB up front to avoid `memory.grow` stalls mid-frame.

### Fixed

- JavaScript: the npm package ships branded prototypes, which fixes "Invalid target type in render" on the website.
- The web gallery and healthcheck initialize WebAssembly with `init({ module_or_path })`.

### Website

- A visual gallery at `/gallery` replaces the old healthcheck pages, and `run_web` opens it by default.
- The homepage gains the hero shader. Astro points at the local package directory so subpath imports resolve.

### Examples

- Rust: the `swirl` shader moves to `examples/rust/examples/shaders/swirl.wgsl`, loads by path, and uses top-level `draw` and `resize`.
- Web: simpler pass dependencies and the `createTextureTarget()` API.

## 0.10.7: Docs pipeline, website, API completeness, and release flow

The largest release so far, shipped as a patch. Most of these features were planned for later releases and landed early to round out the API before the iOS and Android work.

### Geometry and instancing

- Several meshes per pass, each with its own draw call and bind group updates, so meshes in one pass can use different textures.
- Shaders own their meshes. `Shader.add_mesh` checks compatibility on attach against the `@location` inputs that naga reflects from the source, and `Pass.add_mesh_to_shader` attaches through a pass.
- Meshes group by shader, with one pipeline per shader in a pass.
- `Mesh` and `Vertex` build custom shapes, and `Quad` is the first built-in shape.

### Shaders

- Every way to get data to the GPU: vertex and index buffers, storage buffers and their arrays, uniforms and their arrays, textures, storage textures, samplers, and push constants.
- `shader.set("key", texture)` binds a texture in Rust, JavaScript, and Python, with texture and sampler bindings detected from the shader.
- `set("ssbo.*")` writes storage buffer fields at the offsets naga reports, uploaded on the next frame.
- Array elements are addressable by index in storage buffers and uniforms, nested structs and arrays included.
- Push constants run natively for a single root, and fall back to a uniform buffer on the web, over the size limit, or with several roots.
- One fetch helper for native and the web loads `Shader.fetch` and texture URLs.

### Textures and storage

- A public `Texture` handle with a `TextureId`, registered with the renderer.
- `TextureMeta` carries a texture's id plus the dimension, array flag, and class that naga reflects.
- Storage buffers bind as read-only or read-write from their access flags and upload their CPU copy each frame.
- `TextureInput` (bytes, a path, or a clone of a texture) and `TextureOptions` (size, format, sampler), with a `TextureFormat` independent of wgpu.
- `create_texture(input)`, `create_texture_from_file(path)`, and `create_texture_with(input, options)`, plus `create_texture_with_size`, `create_texture_with_format`, `create_2d_texture`, `create_3d_texture`, `create_storage_texture`, and `create_depth_texture`. Web: `createTexture` takes a `Uint8Array`, a URL, or a query selector. Python: `create_texture` takes bytes, a path, or a NumPy array.
- Error types per module replace every `Box<dyn Error>`.

### Rendering

- Surface configuration picks formats and `view_formats`, and window targets recover from failed frame acquisition, with retries centralized in the renderer.
- MSAA with resolve, on window targets and optionally on texture targets. The sample count is negotiated and stored in the render context.
- The pipeline cache keys on shader hash, format, and sample count.
- Pools for transient targets and readback buffers.

### Build system and documentation

- Unit tests for every package before each build, and a git hook that test-builds every platform before a push.
- Scripts that test, compile, and publish JavaScript, Python, and Rust, and a GitHub Actions job that builds every package on every OS.
- Docs live in `docs/api` and reach Rust through `#[lsp_doc]`. The build checks that every object and method has a page with a `## Example` section.
- The website moves into this repository under `docs/website` and is generated from `docs/api`, with JavaScript and Python examples taken from annotated healthcheck scripts.
- After a tag publishes to npm and PyPI, a workflow updates the website and the JavaScript example to the new version.
- Feature parity across languages, checked at build time, with every public item linked to its `docs/api` page.
- API links point at <https://fragmentcolor.org>. Docs examples share one style: an async wrapper with pollster, and no filesystem reads.
- Unused mobile code is gone: `headless()`, `render_bitmap()`, and the platform `FragmentColor` wrappers.

### Platforms

- Scaffolding for iOS and Android, with no generated bindings: `new_ios`, `create_target_ios`, `create_texture_target_ios`, and `render_ios`, the Android equivalents, and the `IosTarget`, `IosTextureTarget`, `AndroidTarget`, and `AndroidTextureTarget` types.

## 0.10.6: JavaScript support

Not published: the build was unstable.

- JavaScript support, published to npm
- Constructors no longer return tuples
- A lazily loaded `Renderer` with a simpler API
- A script that bumps the version
- wgpu and other dependencies updated

## 0.10.5: Python on Windows and Linux

- Python works on Windows and Linux
- Build artifacts are generated and published to PyPI automatically

## 0.10.4: Python import fix

- The PyPI package imports correctly
- `Pass` and `Frame` fixes
- Missing public API methods added

## 0.10.3: Rust 2024 edition

- The crate moves to the Rust 2024 edition
- Small documentation fixes

## 0.10.2: Python support

- A first Python implementation, published to PyPI

## 0.10.1: Cleanup and bug fixes

- A simpler internal shader representation
- A `BufferPool` for uniform buffers
- Runtime errors return instead of panicking
- Uniform getters and setters fixed
- `Renderer::render` takes two arguments: a renderable and a target
- The renderer accepts a `Shader`, a `Pass`, or a `Frame`
- A simpler `Target` interface for the common single-target case
- Helper functions that initialize each platform
- Less boilerplate in the Rust demos

## 0.10.0: The basic API

- `Renderer`, `Shader` with a generic `set()`, `Pass` (render and compute), `Frame`, and `Target`
- Experimental GLSL support

## Before 0.10

Versions up to 0.9.0 were discarded. 0.10.0 is a rewrite from scratch, begun in January 2025.
