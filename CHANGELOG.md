# Changelog

## 0.12.7 (unreleased)

### Renderer and meshes

- `Shader.add_mesh` fails when a later vertex or instance lacks an attribute the shader reads, and the error names the attribute and the index: `color: expected vec4<f32>, got no value at vertex 3`. Only the first vertex and instance were checked, and a later one drew with zeros.
- `Material::pbr` writes light the target can show. On the `Rgba8Unorm` and `Bgra8Unorm` targets FragmentColor creates, which hold display values, it encodes as sRGB, so PBR scenes, docs images and goldens drawn there get brighter: they rendered dark and harsh. An sRGB target encodes on write and a float target keeps linear light, so both get linear light, and nothing encodes twice. The renderer sets the shader's `override ENCODE_SRGB: bool` from the target's format, and a custom shader that declares it gets the same value.
- The renderer releases a texture when the last thing holding it drops: every `Texture` handle, the `TextureTarget` that owns it, and each shader uniform or pass target that binds it. It kept every texture until `unregister_texture`, so a demo that made a new texture on each resize leaked the old ones.
- A pass that draws `Model`s and sets no depth target draws with a depth buffer, so overlapping scenes now draw with depth: the nearest surface shows whatever order the models were added in, where a far model added last covered a near one. The buffer matches the pass's color attachment in size and sample count. The renderer reuses it from frame to frame and frees it a few renders after the target changes size. The shaders in that pass that draw no model, such as a fullscreen backdrop or an overlay, ignore depth and draw in order, so a backdrop drawn first stays behind the models. `Pass.set_depth_target` still sets a pass's own depth texture, and passes without models, such as 2D and fullscreen passes, draw without depth.
- PBR materials light a `Model` scaled differently along each axis, or mirrored, the way its surfaces face. The registry's `mesh_transform_normal` uses the inverse transpose of the model matrix, where the plain matrix tilted normals toward the stretched axis.
- `Renderer.render` returns the errors the GPU raises for a frame in Rust, Python, Swift and Kotlin: validation errors, internal errors such as a shader the backend compiler refuses, and running out of memory. It builds each render and compute pipeline, and records and submits each frame, inside error scopes, and returns what they catch as `RendererError::DeviceError`, labeled with the pipeline, the frame, or the bind group or texture view that failed. Its `kind`, of the new `ErrorKind` enum, is `Validation`, `Internal` or `Memory`, from the scope that caught it. When one error causes another, the first one returns. It returned success, drew nothing and logged an uncaptured error on every frame. A pipeline that fails to build stays out of the cache, so the next frame builds it again and reports the same error. A failed frame is still presented and shows a cleared image, or on Apple GPUs what the passes before a compute submit drew, while a shader without its mesh or an instance count past the mesh's instances fails before `render` takes a frame, so a window keeps its last image. JavaScript gets these errors later, as the JavaScript notes below describe.
- A hidden `Model` draws nothing, and neither does a `Model` mesh in a pass the Model isn't in, as when two passes share one `Material`. Either one lost the whole frame to a GPU error, or drew the mesh at another Model's transform. The skip holds per shader and per pass: a shader that draws any of the pass's Models skips a Model's mesh that no visible Model of the pass draws through it. A shader of your own that draws none of the pass's Models, such as an outline, still draws the mesh when it reads no Model columns.
- `render` of a shader that reads `@location` vertex inputs and has no mesh returns `ShaderError::InvalidKey` naming the input, in all five languages. That covers a shader rendered on its own, one whose meshes were removed, and a `Model` whose attach failed. It built a pipeline wgpu rejected and drew nothing.
- A shader that takes its vertex inputs as `@location` members of a struct argument draws a mesh, in all five languages. Each member reads the mesh attribute of its own name, as an argument does, in `Shader.add_mesh`, `validate_mesh` and `render`. `add_mesh` said a shader whose only inputs were struct members had no vertex inputs and drew fullscreen, and one that also took plain `@location` arguments attached the mesh and failed validation at render.
- A mesh whose index list is empty draws nothing, where `render` panicked with "buffer slice can not be empty".
- `render` returns `MeshError::InvalidInstance` when `set_instance_count` asks for more instances than the mesh holds and the shader reads instance attributes. A shader that reads none, such as one that indexes a storage buffer with `@builtin(instance_index)`, draws any count. A shader that draws none of the pass's Models and reads the Model columns of a `Model`'s mesh gets the error too, whether that Model is visible or not: a Material's shader rendered on its own, or an outline of your own that follows the Model's transform. Its message names the columns the shader reads and says that no Model in the pass draws the mesh with that shader. It lost the frame to a GPU error.
- `Pass.set_clear_color` hands each channel to the GPU as given, in every language. A float target such as `Rgba16Float` clears to `[4.0, -1.0, 0.25, 1.0]`, and a unorm target clamps on the GPU. Each channel was clamped to 0 to 1 and truncated to 8 bits first, so 0.5 cleared to 127/255.
- `render` checks each render pass against the attachment it draws into before the pass records, in every language. A depth target whose width or height differs from the color attachment's returns the new `RendererError::DepthSizeMismatch`, which names both sizes, as a different sample count returns `DepthSampleCountMismatch`. A shader whose `@location(0)` output the format can't take, such as `vec4<f32>` into an `Rgba32Uint` target, returns `PassError::InvalidColorTarget`, and so does `AlphaMode::Blend` into a format the device can't blend, such as an integer format, or from a shader that writes fewer than four channels, since blending reads the alpha. In 0.12.6 each one lost the frame: `render` returned success, and the GPU's error reached only the log, or `console.error` on the web. The browser refused a blended output without alpha that native drew. A pass with a target of its own fails before `render` takes a frame, and so does a viewport with a side past the device's largest 2D texture, which returns the new `PassError::ViewportLimit`. 0.12.6 shrank that side to the limit and squeezed the image to another zoom.
- The device opens with float32 blending wherever the GPU offers it, so `AlphaMode::Blend` into an `Rgba32Float` target draws there. A device without it refuses the pass with `PassError::InvalidColorTarget`. In 0.12.6 every device lost the frame.
- A shader whose fragment entry point writes no color, such as a depth-only pass, draws with no color channel written, as WebGPU requires, so the target keeps its color. The browser refused its pipeline in 0.12.6, and native drew it.
- A pass that binds a texture it draws into, such as a shader that samples its own render target or reads the pass's depth target, returns the new `PassError::TextureConflict` in every language, and so does a compute shader that reads and writes one texture in one dispatch. The message names the binding and says to use the texture in a later pass, which still works. In 0.12.6 the frame was lost and `render` returned success.
- A depth target that `unregister_texture` removed, or that another `Renderer` created, makes `render` return `TextureNotFoundError`, as a color target does. In 0.12.6 an unregistered depth target drew without depth and said nothing. Texture ids were counted per `Renderer` then, so another Renderer's depth texture drew with whichever texture this one held under the same id, or without depth.
- A PBR map set to a texture that `unregister_texture` removed, or that another `Renderer` created, makes `render` return `TextureNotFoundError`, as every other texture uniform does. In 0.12.6 the slot drew its 1x1 default without a word.
- A `Renderer` refuses a target another `Renderer` created with the new `RendererError::ForeignTarget`, before it takes a frame. In 0.12.6 Rust, Python, Swift and Kotlin panicked inside wgpu, which stopped the app, and the web lost the frame with only a `console.error`. A render of compute passes alone draws into no target and still runs. A `Renderer` that has no context yet returns `ForeignTarget` for such a target too, where it returned `NoContext`.
- Calls that race to create a Renderer's first target or texture, such as a `Promise.all` of two `create*` calls in JavaScript, share one device. In 0.12.6 each opened a device of its own and the Renderer kept the last one, so a target made by another call couldn't be drawn.
- A compute dispatch with more workgroups in a dimension than the device takes, 65535 on most GPUs, returns the new `PassError::DispatchLimit` before anything runs. In 0.12.6 the whole render was lost and `render` returned success.
- `Pass.set_target` and `Pass.set_depth_target` refuse a texture with a mip chain or with layers with `PassError::InvalidColorTarget`, in every language. A pass draws into one level of one layer, so in 0.12.6 such a target lost every frame while `render` returned success. `create_texture` builds a mip chain unless `mipmaps` is false.
- Lights are clustered, and a `Material::pbr` shader takes as many as the device allows, in every language. At each render the renderer sorts every light with a `range` into a grid of cells over the pass's camera view, with a compute pass, and each fragment shades only the lights of its own cell plus the lights without a range. A still scene sorts once, and a moved light or camera sorts again. A new color or intensity sorts nothing and uploads only the lights from the first one that changed to the last. Every OpenGL backend, WebGL2 included, takes the lights through a uniform block the size the device allows, from 253 to 1,021 lights, and loops over all of them, and so does a device without compute shaders or storage buffers in fragment shaders. A custom shader gets the same lights by declaring the `lights` and `light_grid` roots that the `Material::custom` page lists.

### Shaders

- `Renderer.render` checks each shader against the device before it takes a frame, in every language, the web included, where the GPU's error came too late to throw from the call. A group past `max_bind_groups`, a binding past `max_bindings_per_bind_group`, more bindings of a kind in one stage than the device takes, a uniform or storage buffer past its binding size limit, and a workgroup past the device's sizes, invocations or workgroup memory (an override's default included) return the new `RendererError::LimitError { item, limit, value, max }`. A compute shader on a device without compute, storage a stage of the device can't bind, a storage format or access it lacks, and anything else the device's own shader validator refuses return the new `RendererError::FeatureError { item, feature }`. A render shader without its `@vertex` or `@fragment` entry point returns the new `RendererError::EntryError { stage }`, where the pipeline looked for `vs_main` and `fs_main`, as with a registry helper such as `sdf2d/circle` rendered on its own. Each one raised a GPU error, cached a broken pipeline and drew nothing on every frame. JavaScript throws an `Error` with the fields as properties, Python raises `LimitError`, `FeatureError` and `EntryError`, each a `FragmentColorError` with the fields as attributes, Swift throws `FragmentColorError.Limit`, `.Feature` and `.Entry`, and Kotlin the `FragmentColorException` subclasses of those names. A shader source is checked once per renderer, so a refused one returns the same error each frame without being checked again, and the checks read the device's features and limits once per renderer.
- A texture uniform is checked against its slot before the frame, by the rules wgpu binds with, so `Shader.set(name, texture)` and the `Material` map setters given a texture fail the same way in every language: a depth texture, a 3D texture or an unfilterable float in a sampled 2D slot, a texture without `STORAGE_BINDING` in a storage slot, a storage texture of another format or with a mip chain, and a multisampled texture in a single-sampled slot. These return the new `RendererError::BindingError { name, expected, found }`: Python `BindingError`, Swift `FragmentColorError.Binding` and Kotlin `FragmentColorException.Binding`. A slot never set returns the same error, and a texture `unregister_texture` removed returns `RendererError::TextureNotFoundError`. An unset PBR map slot of a render shader binds its 1x1 default only where its slot takes a 2D float texture, and a compute shader, which binds no default, needs its texture set. A cube, cube array or 2D array slot takes no FragmentColor texture yet, and says so. On the web these drew a broken frame and logged a GPU error, and on native the error didn't name the slot. A compute shader skipped an unset texture without a word.
- Each binding is visible to the stages whose entry points use it. A fragment entry point writes a storage buffer or storage texture, and reads a storage texture, where the binding was visible to compute only. Bindings used by one stage count against that stage's limits only.
- Every storage texture format binds as the format the shader declares. `r32float`, `r32uint`, `r32sint`, `rg32*`, `rgb10a2uint`, `r64uint` and the 16-bit norm formats bound as `Rgba8Unorm`, which every pipeline that used them failed, and `read_write` `r32float`, the format WebGPU guarantees for it, never worked. A `texture_1d` binds as a 1D view, and a multisampled depth texture as one.
- Immediate data works in compute shaders: one root that fits takes native immediates, and more roots take a uniform group, as in render shaders. A compute shader that read a root failed to build its pipeline and bound the root over its group 0, binding 0.
- The group that holds immediate roots as uniforms, on the web and on devices without immediates, takes the lowest index the shader leaves free, where it took the one after the highest, so a shader that uses groups 0 and 3 still fits a 4-group device.
- A compute shader that leaves a lower group empty gets an empty group there, as a render shader does, where its pipeline layout put the next group in its place and failed.
- `@size` and `@align` on uniform, storage and immediate structs keep their layout on the browser's WebGPU. The WGSL wgpu writes for the browser drops them, so a struct that used them read shifted data or failed validation there. FragmentColor hands the browser WGSL with a `@size` on each member of such a struct, which pins every offset and the struct's size, under a name naga's writer keeps, so struct names that end in a digit or an underscore, or that aren't ASCII, work too. The GL backends, WebGL2 and native GL, write such a struct into a std140 block member by member, which loses the layout, so `Renderer.render` refuses it there with `RendererError::FeatureError`, where it read shifted data with no error.
- `Shader.new` refuses what no device FragmentColor opens can run, in all five languages: features outside the set it requests (`f16`, `primitive_index`, subgroups, `texture_external`, texture atomics), two globals on one `@group` and `@binding`, a `lights` root included, writable storage in a `@vertex` entry point, an immediate root that breaks the uniform layout rules, which the web and devices without immediates bind it by, and a filtering `sampler` that samples a depth texture, which the web refuses. Each one built a shader that failed at every render, or on the web.
- `Shader.new` refuses two vertex inputs with one name, such as `position` members of two struct arguments, with an error that names the input and both locations. Both read the one mesh attribute of that name, so the shader drew wrong with no error.
- `Shader.new` of a uniform holding an array of structs builds one key per field, not one per element. A large array built a key for each element, and an 800 MB uniform ran the process out of memory. `list_keys` no longer lists the elements, and `set` and `get` still take them by index, as in `u.items[2].v`.
- `Shader.set` with raw bytes on a storage or immediate root takes exactly the root's size and throws for any other length, in every language. More bytes were cut off and fewer were padded with zeros, with no error.
- `Renderer.load`, and `render` on native, keep a `Material` map that fails to upload queued and bind the maps that upload, in every language. The first render returned the error and dropped every map it had taken from the queue, and later renders drew the 1x1 defaults with no error. Each `load` and native `render` now returns the error until the input is fixed or replaced. A map the shader doesn't declare fails the render that uploads it, once, and no longer drops the maps queued after it. The latest setter call for a map wins, even one made while `load` uploads an older input, which is dropped instead of binding over it.
- On WebGL2 and native GL, an immediate root binds as a uniform padded to a multiple of 16 bytes, where a root such as a `vec3<f32>` failed its pipeline on every frame.
- A uniform upload past 64 KiB reuses its buffer on each render, where every frame created a new buffer that was never freed.

### Textures

- `Texture.resize(size)` gives a texture a pass draws into a new size, in Rust, JavaScript, Python, Swift and Kotlin. Every clone of the handle follows, and so do the shader uniforms and pass targets that hold it, so a depth texture can follow a window without being created again. A new size starts with blank pixels. A texture without `RENDER_ATTACHMENT` usage returns `TextureError::FixedSize`, and a width or height of zero returns `TextureError::InvalidInput`. Both keep the old size.
- A `Texture` from `TextureTarget.texture()`, and every clone of a `TextureTarget`, follow `TextureTarget.resize`. They kept the old size, and `write` and `get_image` reached the old texture.
- `create_texture` decodes an HDR file into `Rgba16Float`, and a decoded image fills an explicit `Rgba16Float` or `Rgba32Float` format. Each uploaded RGBA8 bytes into a row two or four times as wide and raised a GPU validation error, leaving the texture blank. A value past the half float range saturates at 65504 instead of becoming infinity, and an explicit `Rgba32Float` keeps the full range, at every mip level.
- `Rgba16Float` and `Rgba32Float` get the full mip chain other formats get, from a decoded image, raw bytes, `Mipmap.build` and a whole `Texture.write`, so a KTX2 half float texture with baked levels takes a whole write. A box filter averages each level from the f32 values of the level above, so values below 0 and above 1 survive at every level. In `Rgba16Float` those values are unsaturated, and a level saturates only when it is stored. Raw bytes upload as given at level 0, an infinity or a NaN payload included.
- `Mipmap.build` reads a file's format from its bytes, as `create_texture` does, so a file name without an extension works.
- A decoded image in `Bgra8Unorm` stores blue first in `create_texture` and `Mipmap.build`, so its red and blue no longer swap. A JavaScript `ImageData` or canvas does the same in `createTexture`, the Material texture setters, `Texture.write`, `writeRegion`, `Mipmap.build` and the `createStorageTexture` seed. A NumPy array is raw bytes in the named format and keeps its order.
- `Texture.write` rebuilds the mip chain that `create_texture` builds by default, so a texture sampled far away shows the new image instead of the first one.
- `create_texture` reads an image's size from its header and checks it against the device before the pixels decode, so an image too large to upload takes no memory.
- Creating, writing and reading back a texture stays clear of GPU validation errors. The checks run before any wgpu call and return `TextureError::InvalidInput`, the error `resize` returns: a usage bit outside the five `TextureUsage` flags, a usage the format can't take on the device, such as `STORAGE_BINDING` on `Depth32Float`, a depth texture with layers, pixels for a depth texture, and a write or readback of a block-compressed KTX2 texture. A readback that needs a buffer past the device's largest, or a depth readback on GLES or WebGL2, returns `TextureError::Readback`.
- A KTX2 file that lists more mip levels than a chain down to 1×1 holds returns `TextureError::InvalidInput`, which names the size and both counts. The extra levels were dropped without a word.
- A block-compressed KTX2 chain down to 1×1, the chain encoders write, uploads its levels smaller than one block. Each one raised a GPU error and stayed blank, since the copy asked for part of a block.

### Scene

- `Scene.load` refuses a malformed glTF file with `SceneLoadError::Invalid`, which names the accessor or node, in every language. A `byteStride` shorter than one element, and an index, `TEXCOORD` or `COLOR` type the glTF reader doesn't take, panicked inside the reader in every build: Rust panicked, the web build aborted, and Python, Swift and Kotlin raised a panic exception instead of the scene error. A node that is its own ancestor overflowed the stack and aborted the process. A count of 0, and a `POSITION`, `NORMAL` or `TANGENT` type the loader doesn't take, panicked only in debug builds. Release builds read garbage or nothing there, and an empty index list crashed the next render.
- `Scene.load` refuses, with the same error, two things glTF allows but the loader can't take: a mesh attribute or index accessor with more elements than 32-bit indices reach, and a sparse `POSITION` or index accessor without a buffer view that spans more bytes than all of the file's buffers hold. A file of a few hundred bytes could declare billions of zeros that way, and the loader panicked, aborted or ran out of memory filling them. On the web the glTF parser refuses a count past 32 bits itself, as `SceneLoadError::Gltf`. Accessors the loader doesn't read, such as animation tracks, aren't checked, though the parser still refuses any accessor with neither a buffer view nor sparse values, which glTF reads as all zeros.
- `Scene.load` draws glTF triangle strips and fans, expanded into triangle lists. It drew them as lists, which scrambled the geometry.
- `Scene.load` keeps a glTF file's 16-bit PNG textures and its gray PNGs with alpha, converted to 8 bits per channel. It skipped them with a log warning, and the slot kept its 1×1 default.
- `Camera.perspective`, `Camera.orthographic`, `Camera.set_aspect` and `Camera.look_at` refuse parameters that leave no usable view. `perspective` refuses a value that isn't finite, a field of view at or below 0 or at or past π, an aspect of 0, a near or far plane at or below 0, and equal near and far planes. `orthographic` refuses a plane that isn't finite and a frustum with no width, height or depth. `set_aspect` refuses 0, NaN and infinity. `look_at` refuses a vector that isn't finite, a target on the camera's position, a zero up, and an up on the line of view (a sine below 1e-12). Any of them also refuses parameters whose matrix overflows `f32`. Rust returns the new `CameraError`, JavaScript throws an `Error`, Python raises `CameraError` (a `FragmentColorError` and a `ValueError`), Swift throws `FragmentColorError` and Kotlin throws `FragmentColorException`, and the camera keeps its old state. Most of these filled the matrix with infinities or NaN, an exactly parallel up among them, and every model drawn through the camera vanished without an error. The rest drew a flipped or collapsed image, or one with no usable depth range, such as a near plane of 0. `Scene.load` reports such a glTF camera as `SceneLoadError::Camera`, which names the camera, and one whose `zfar` isn't past its `znear` as `SceneLoadError::Invalid`.
- `Camera.look_at` builds its view in double precision from a normalized up. An up nearly along the view gets the roll its numbers describe, where `f32` rounding picked it, and a tiny up no longer leaves axes that aren't unit length.
- A glTF perspective camera without `zfar` gets the infinite projection glTF asks for. It got a far plane of 1e6, which clipped anything farther, filled the matrix with infinities for a `znear` of 1e6, and gave a reverse-Z projection past it.
- A spot light with equal inner and outer cone angles draws the hard edge its docs promise on every GPU. The PBR shader handed both edges to `smoothstep`, which WGSL leaves undefined when they meet.
- `Material::pbr` lights a pixel whose light lies along its view ray, such as a light at the camera, on every GPU. Rounding could carry the Fresnel term's dot product past 1, and `pow` of the negative base that left is undefined in WGSL. GPUs that return NaN for it, lavapipe among them, drew those pixels black.
- Each of the five PBR texture slots samples its own UV set through its own `KHR_texture_transform`, in Rust, JavaScript, Python, Swift and Kotlin. `Material.texture_uv(slot, set)` picks `UV0` or `UV1` for a slot, and `Material.texture_transform(slot, offset, scale, rotation)` sets its transform. The new `TextureSlot` enum names the slot: `BaseColor`, `MetallicRoughness`, `Normal`, `Occlusion` or `Emissive`. A set other than 0 or 1 returns the new `MaterialError::UvSet` in Rust; JavaScript throws an `Error`, Python raises the new `MaterialError` (a `FragmentColorError` and a `ValueError`), Swift throws `FragmentColorError` and Kotlin throws `FragmentColorException`. `uv_transform` still sets one transform on all five slots. Every slot sampled `UV0` through that one transform.
- `Scene.load` reads the `texCoord` and the `KHR_texture_transform` of every texture slot, the extension's own `texCoord` included, so an occlusion map on `TEXCOORD_1` samples `UV1`. The loader read the transform of the base color map alone, applied it to every map, and logged a warning for the metallic-roughness and emissive transforms. A primitive without `TEXCOORD_1` takes its `TEXCOORD_0` values as `UV1`, where it got `[0, 0]`. A `null` `KHR_texture_transform`, which glTF doesn't allow, counts as none on every slot.

### JavaScript

- `createStorageTexture`, `Mipmap.build`, `Material.alphaMode` and `Mipmap.level` throw an `Error` for a format, mode or index that isn't a whole number of the right kind. `undefined`, `NaN` and strings read as `R8Unorm` or `Opaque`, and fractions truncated, so `level(1.5)` returned level 1.
- `createTexture` and the Material texture setters read an options object by its keys. An object with `width` and `height` beside `format` or `usage` read as a bare size and dropped the rest. It now throws and asks for the size under `size`. A bare size that fails to read, such as `[1.5, 2]` or `{ width: 1.5, height: 2 }`, throws too, where it was dropped and the texture uploaded with the default options.
- A numeric `compare` in sampler options must be a whole number from 1 to 8, so `1.5` throws instead of reading as `Never`.
- `Vertex`, `Instance`, `Quad` and `TextureId` carry the `__fc_kind` brand the other classes have. Minifiers rename classes, and the checks that passed these objects into the library fell back to the class name, so a production build failed where a development build worked: `mesh.addVertices` rejected its `Vertex` objects. The checks read only the brand now, and a build fails when a type that crosses into the library has none. A site that keeps class names in its build to work around this, as the FragmentColor website does with `keepNames` in `docs/website/astro.config.mjs`, can drop that once it loads 0.12.7.
- A GPU error from a frame throws as an `Error`. WebGPU reports it after `render` has returned, so the frame that caused it can't throw it: the next `render` does, before any work. `load`, `readTexture`, `readStorage`, and `getImage` on a `Texture` or `TextureTarget` wait for earlier frames to report first, so they throw the error even when called right after the frame. A call that waits throws only the error of a frame rendered before it began, and the error of a frame rendered while it waits, even one rendered in the same task right after the call, stays for the next call. The thrown `Error` carries the error's `kind`, an `ErrorKind`, its `label` and the device's own `detail` as properties, and its `message` holds the full text. Only the first error to land throws, once, and every error still logs a `console.error` as it lands, thrown or not. A pipeline that failed stays cached until its error lands and then leaves the cache. In a loop that renders each frame, the next `render` throws the error before any work, and the one after it builds the pipeline again and reports the same error. These errors only went to `console.error`, and `render` kept returning `undefined`.
- Setting `size.width`, `size.height` or `size.depth` checks the number as `new Size` does and throws an `Error` for a negative, NaN or out-of-range one, keeping the old value. The setters wrapped it modulo 2^32, so `size.width = 2 ** 32 + 64` made a 64-pixel-wide target.
- `Pass.setClearColor` declares its argument as `number[] | Float32Array` in the TypeScript types, where it was `any`.
- `Pass.setClearColor([1.2, 0.5, 0.25])` clears to the color every other language gets. One channel above 1 switched all of them to a 0 to 255 scale, so this color cleared near black. A channel that isn't a number, such as `undefined` or `'1'`, throws an `Error` where it became 0.

### Python

- Draws fire with rendercanvas 2.7, where `canvas.request_draw` never called the draw function and `examples/python/main.py` showed nothing. `Renderer.create_target` registers a rendercanvas context on the canvas, and `render` presents each window frame itself, from a draw function or a timer.
- A hidden or minimized window raises `OccludedSurfaceError`, a subclass of `FragmentColorError`, so a render loop can skip the frame without reading the message. Rust reports it as `SurfaceError::Occluded`.
- A GPU error that `render` returns raises `DeviceError`, a subclass of `FragmentColorError`, whose `kind` attribute holds an `ErrorKind`, `label` names the step that failed and `detail` holds the device's own words. `str(e)` gives the full text.
- `create_texture` with a NumPy array honors `format`. An array whose channels match the format's bytes per pixel uploads as it is, so a gray `(H, W)` array fills `R8Unorm`, and 1 or 3 channels gain an opaque alpha for an 8-bit RGBA or BGRA format. The array was always expanded to RGBA8 first, so `R8Unorm` uploaded the expanded bytes. A side past `u32` is refused before the bytes are collected, where it truncated to a small texture.
- `size()` is a method on `Texture`, `TextureTarget` and `WindowTarget`, as in every other language and like every other Python getter. `TextureTarget.size` and `Texture.size` were properties, and `WindowTarget.size()` the one method among them.
- Counts, indices, sizes and regions take an int and raise `InvalidInputError`, a `FragmentColorError`, for one below 0 or past `2**32 - 1`, however large, as JavaScript throws for one. That covers `Pass.set_compute_dispatch`, `Pass.set_viewport`, `Mesh.set_instance_count`, `Mesh.set_indices`, `Scene.get_pass`, `Scene.add_to`, `Mipmap.level`, the list and dict regions of `Texture.write_region`, `Size(...)` and its fields, every `[w, h]` or `{ "width": ... }` size, and `ScreenRegion(...)` with `from_region` and `from_tuple`. They raised `OverflowError`, or a `TypeError` about a failed enum for a list. A NumPy integer counts as an int, a float, a `bool` or a string raises `TypeError`, and a list of the wrong length raises `InvalidInputError`.
- `Pass.set_clear_color` raises `InvalidInputError`, a `FragmentColorError`, for a list that isn't 3 or 4 numbers long, with a message that names the call, where it raised a `TypeError` about a failed enum.
- `Pass.set_target` and `Pass.set_depth_target` raise `InvalidInputError`, a `FragmentColorError`, for a texture they refuse, such as one without `RENDER_ATTACHMENT` usage or with a mip chain. `render` raises the same class for a pass error that names a value: a target format the shader can't fill, a blend the target can't take, and a viewport or compute dispatch past the device's limit.
- `Renderer.read_texture` and `Renderer.unregister_texture` take a texture id as an int of any size, and raise `InvalidInputError` for one below 0 or past `2**64 - 1`, where they raised a `TypeError` about a failed enum.
- Number errors name the parameter that failed, and the item for a list, as JavaScript's do: `Mesh.set_indices: item 3 must be a whole number from 0 to 4294967295, got -1`.
- Every color and vector argument reads the same way. The colors are those of `Pass.set_clear_color`, `Light.set_color`, `Light.point`, `Light.directional`, `Light.spot`, `Scene.ambient`, `Material.base_color` and `Material.emissive`. The vectors are the positions and directions of `Light.point`, `Light.directional`, `Light.spot`, `Light.set_position` and `Light.set_direction`, the vectors of `Model.translate` and `Model.scale`, the `Model.rotate` axis, and the three vectors of `Camera.look_at`. Each takes any sequence of numbers, NumPy arrays included. A bytes form (`bytes`, `bytearray`, `memoryview` or a uint8 ndarray) or a `bool` item raises `TypeError`, and an item that is NaN, an infinity or past the 32-bit float range, such as `10**400` or `1e39`, raises `InvalidInputError`, as does a list of the wrong length. Each error names the parameter. A bytes form read as numbers, a huge int raised `OverflowError`, and a vector of the wrong length raised pyo3's builtin `ValueError`.
- `create_storage_texture`'s `usage` and `usage_bits` raise `InvalidInputError` for an int out of range, where they raised `OverflowError` or a `TypeError` about a failed enum.
- `examples/python/main.py` and `multiobject.py` round the rendercanvas resize size to whole pixels, which `resize` now requires.
- `Material.uv_transform` and `Material.texture_transform` raise `InvalidInputError` for an offset or scale that isn't two numbers long, where pyo3 raised a builtin `ValueError` that `except FragmentColorError` missed. A string raises `TypeError`, and an item that is NaN, an infinity or past the 32-bit float range raises `InvalidInputError`, as in every binding.
- `Model.set_transform` raises `InvalidInputError` for a matrix that isn't 16 numbers or 4 columns of 4, naming the call and the column, where pyo3 raised a `TypeError` about a failed enum. A string or a `bool` raises `TypeError`.
- The type stub takes what the bindings take. A parameter hinted `int` is `SupportsIndex` and one hinted `float` is `SupportsFloat`, so NumPy scalars such as `np.uint32` and `np.float32` fit. A list of numbers also takes an ndarray, and a dict parameter is a `Mapping`, so pyright no longer rejects calls that run, such as `Light.point(np.zeros(3), [1, 1, 1])` or `Pass.set_compute_dispatch(np.uint32(64), 1, 1)`.

### Swift

- A GPU error that `render` returns throws `FragmentColorError.Device(kind:label:detail:)`, with an `ErrorKind`, the step that failed and the device's own words. Every `FragmentColorError` case now has the error's full text as its `description` and `localizedDescription`, where they printed the enum case.
- `TextureTarget.resize([w, h])` and `WindowTarget.resize([w, h])` throw `FragmentColorError` for an array that isn't 2 or 3 numbers long, where they stopped the app. So does `TextureRegion.from(_:)` for an array that isn't 2, 4 or 6 numbers long, as in Kotlin, so a call to it needs `try`.
- `createTarget` checks that the layer address it receives names a `CAMetalLayer` and throws `FragmentColorError` for any other object. A plain `CALayer` passed to the raw uniffi method aborted the app. A drawable side that is negative, NaN, infinite or past `UInt32.max` throws too, where a negative or NaN side became 1 pixel and an infinite or larger one became `UInt32.max`. A fraction rounds down, and a side below 1 gets 1 pixel.
- `setViewport` and the `ScreenRegion(origin, size)` inits are `throws`. A list of the wrong length throws `FragmentColorError`, where they stopped the app with `precondition`, and so do a side of 0 and a far edge past `UInt32.max`.
- `setClearColor` takes `[Double]` as well as `[Float]`, the forms every other color setter takes.
- A list of integer literals compiles wherever a `[Float]` or `[Double]` list goes, such as `setClearColor([0, 0, 0, 1])`, `Light.point([0, 2, 0], [1, 1, 1])` and `model.translate([0, 1, 0])`, and takes the `[Float]` overload. With both overloads to pick from, the colors and vectors of `Light`, `Scene`, `Material`, `Model`, `Camera` and `Mesh` failed to compile with "ambiguous use".
- The `Camera`, `Light` and `Material` setters that return their receiver, such as `lookAt`, `setAspect` and `setDirection`, are `@discardableResult`, as `Scene` and `Pass` were. A per-frame update no longer needs `_ =` in front.

### Kotlin

- A GPU error that `render` returns throws `FragmentColorException.Device(kind, label, detail)`, with an `ErrorKind`, the step that failed and the device's own words. Every `FragmentColorException` has the error's full text as its `toString()`, and its `message` lists the fields, such as `kind=VALIDATION, label=frame, detail=...`.
- `setClearColor` takes a `FloatArray`, a `DoubleArray` and a `List<Double>` as well as a `List<Float>`, the forms every other color setter takes.

### Breaking and behavior changes

- `Renderer.render` refuses, before any GPU work, a shader or texture uniform the device can't take, with the errors the Shaders notes list. A shader that rendered nothing with a GPU error now returns an error, and an unset texture uniform is an error in compute passes too.
- `RendererError` gains `LimitError`, `FeatureError`, `BindingError` and `EntryError`, Python the exception classes of those names, and Swift's `FragmentColorError` and Kotlin's `FragmentColorException` the `Limit`, `Feature`, `Binding` and `Entry` cases. A `switch` over every Swift case needs the new ones.
- A texture uniform with no texture is an error, except an unset PBR map slot of a render shader whose slot takes a 2D float texture, which binds its 1x1 default.
- `Shader.new` refuses features no device FragmentColor opens supports, two globals on one binding (a `lights` root included), writable storage in a vertex entry point, an immediate root a uniform can't hold, a filtering sampler that samples a depth texture, and two vertex inputs with one name.
- A `Material` map that fails to upload fails every native `render`, and every `load`, until its input is fixed or replaced, where only the first render failed and later ones drew the 1x1 default.
- The GL backends refuse a uniform, storage or immediate struct whose layout `@size` or `@align` sets.
- `Shader.set` with raw bytes throws unless they fill the whole storage or immediate root.
- `Shader.list_keys` doesn't list array elements.
- A binding no entry point uses is visible to the compute stage only.

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
- `render` returns an error for a frame or pipeline the GPU rejects, where it returned success. Rust returns `RendererError::DeviceError`, Python raises `DeviceError`, Swift throws `FragmentColorError.Device`, and Kotlin throws `FragmentColorException.Device`. JavaScript throws an `Error` from the next `render`, `load`, `readTexture`, `readStorage`, or `getImage` on a `Texture` or `TextureTarget`, and still logs it.
- `RendererError::ValidationError { label, message }` is gone. `RendererError::DeviceError { kind, label, detail }` replaces it, and its text reads `Device error (validation) in 'frame': ...`, with the kind. 0.12.6 returned only bind group and view errors that way, all as validation errors, and its text read `GPU validation error while building '<label>': ...`. A `match` on the old variant and code that reads the old prefix need the new ones.
- `TextureError` has a new `DeviceError` variant, with the fields of `RendererError::DeviceError`. On wasm, `get_image` on a texture or texture target returns it for an earlier frame's GPU error, with the kind, label and detail `render` would report.
- Swift's `FragmentColorError` has a new `Device(kind:label:detail:)` case and Kotlin's `FragmentColorException` a new `Device` class, so a `switch` or `when` that lists every case needs a new one.
- `render` of a shader that reads vertex inputs and has no mesh returns `ShaderError::InvalidKey` in every language, where it returned success. JavaScript throws an `Error`.
- An instance count from `set_instance_count` past the instances a mesh holds is an error from `render` in every language when the shader reads instance attributes, where it reached a wgpu validation error.
- `ScreenRegion` loses its `i32` conversions, along with `from_region_i32`, `from_tuples_i32` and `from_arrays_i32`. They moved a negative origin to 0 and shrank the region to match, so `set_viewport((-100, -100, 400, 300))` drew a different rectangle. A tuple such as `(10, 10, 100, 100)` now reads as `u32`, and a negative number fails to compile, as it does in Swift and Kotlin.
- `Pass::set_clear_color` returns `Result<(), PassError>`, and a channel that is NaN or infinite fails with the new `PassError::Color` and leaves the pass unchanged. NaN and negative infinity became 0, and positive infinity became 1. JavaScript throws an `Error`, Python raises `InvalidInputError`, Swift throws `FragmentColorError`, and Kotlin throws `FragmentColorException`.
- `Color` holds four `f32` channels where it packed four bytes into a public `u32`, so `Color(0xFF0000FF)` and `u32::from(color)` are gone. `Color` no longer implements `Hash`.
- `Color` loses `From<u32>`, so `Color::from(0xFF8000FFu32)` fails to compile. No binding reads packed bytes as a color. Write the channels as floats, such as `[1.0, 0.5, 0.0, 1.0]`.
- `Color` loses its tuple conversions, `From<(f32, f32, f32)>` and `From<(f32, f32, f32, f32)>` in both directions, so `pass.set_clear_color((0.1, 0.2, 0.3))` fails to compile. Pass `[0.1, 0.2, 0.3]`, the list every binding takes.
- `Color` loses its conversions to and from `wgpu::Color`, a wgpu type in the public API.
- `Color::into_vec4_gamma` is gone. Nothing called it, and with the clamp gone it would return NaN for the negative channels a `Color` now keeps.
- Swift's `setClearColor(r:g:b:a:)` and Kotlin's `setClearColor(r, g, b, a)` are gone. A color is a list in every language, as JavaScript and Python already required. Pass `[r, g, b, a]` in Swift and `listOf(r, g, b, a)`, a `FloatArray` or a `DoubleArray` in Kotlin.
- A `ScreenRegion` whose far edge would pass `u32::MAX` is an error in every language, with the new `ScreenRegionError::Overflow` that names the size asked for. It shrank to fit, or got a width of 0. `ScreenRegion::from_tuples` and `from_pixel` return `Result`, the two-corner conversions are `TryFrom`, and `encompass` returns `Result<(), ScreenRegionError>` and keeps the region for a pixel at `u32::MAX`.
- JavaScript's `Pass.setClearColor` takes `[r, g, b]` or `[r, g, b, a]` as an array or a `Float32Array`, the forms every other language takes. A CSS string such as `'#ff8800'`, an `{ r, g, b, a }` object, an `Int32Array` and a `Uint32Array` throw an `Error`. Rust keeps `Color::from_css`.
- A color, a position, a direction, the `Model.rotate` axis, the vectors of `Model.translate`, `Model.scale` and `Camera.look_at`, and a UV offset or scale refuse an item that is NaN, an infinity, or a finite number past the 32-bit float range, in JavaScript, Python, Swift and Kotlin. The error names the call and the item, such as `Light.point: position: item 0 must be a finite number that fits a 32-bit float, got 1e39`. JavaScript throws an `Error`, Python raises `InvalidInputError`, Swift throws `FragmentColorError` and Kotlin throws `FragmentColorException`. All four passed NaN and infinity on and turned a finite number past the range into infinity, so a model or light built from it vanished or lit nothing. A clear color's NaN, which `Pass::set_clear_color` refused already, now fails with a message that names the call. Rust's other setters take the numbers they are given.
- JavaScript's color and vector setters take an array or a `Float32Array`, as `Pass.setClearColor` did already. A `Float64Array`, an `Int32Array` or another array-like throws an `Error`, where it was read as numbers.
- Python raises `InvalidInputError` for a color or a vector of the wrong length, such as `Light.set_color([1, 1])` or `Model.translate([0, 1])`, where pyo3 raised its builtin `ValueError`. `InvalidInputError` is no `ValueError`, so `except ValueError` stops catching it. Catch `FragmentColorError` or `InvalidInputError`.
- Python raises `InvalidInputError` where pyo3 raised a builtin `OverflowError` or `TypeError` for a value of the right type it couldn't use: a count, index, size or region below 0 or past its range, a texture id out of range, a `create_storage_texture` usage out of range, a clear color of the wrong length, and a `Model.set_transform` matrix of the wrong shape. `InvalidInputError` is neither, so `except OverflowError` and `except TypeError` stop catching these. Catch `FragmentColorError` or `InvalidInputError`.
- Python refuses a `bool` where it takes an int or a list of numbers: `set_compute_dispatch(True, 1, 1)` and `set_clear_color([True, 0, 0])` raise `TypeError`, where `True` read as 1.
- Python counts, sizes and regions take ints. A float raises `TypeError`, even a whole one such as `2.0`, where 0.12.6 read a size list of floats and rounded each one down. Sizes and regions that can't be read raise `InvalidInputError`, the subclass `resize` raises for a bad size, where they raised the base `FragmentColorError`.
- JavaScript sizes and regions take whole numbers, as counts do. `new Size(1.5, 2)`, a `Size` setter, `new ScreenRegion`, `setViewport`, `createTextureTarget`, `resize` and the texture regions throw an `Error` for a fraction. This replaces the 0.12.6 rule that sizes and regions take finite numbers from 0 up, under which a fraction rounded down.
- A `ScreenRegion` or a viewport with a width or height of 0 is an error in every language. A zero side kept the pass's clear and skipped its draws without a word. Rust's `ScreenRegion::new`, `from_region`, `from_size` and `from_tuple` return `Result<ScreenRegion, ScreenRegionError>` with the new `ScreenRegionError::Empty`, and the tuple and array conversions are `TryFrom`. `Pass::set_viewport` returns `Result<(), PassError>` through the new `PassError::Viewport`, and it checks a region built field by field as well. JavaScript throws an `Error`, Python raises `InvalidInputError`, Swift throws `FragmentColorError`, and Kotlin throws `FragmentColorException`, from `setViewport` and from the `ScreenRegion` constructors. A pass with no viewport still draws on the whole target.
- `ScreenRegion::from_size_f32`, `ScreenRegion::from_inbound_radius` and the conversions between `ScreenRegion` and `wgpu::Extent3d` are gone. Nothing called them, and the float constructors cut a fraction off without an error.
- JavaScript's `ScreenRegion` drops its `min_x`, `min_y`, `max_x` and `max_y` properties, whose setters skipped every check. Read `minX`, `minY`, `maxX` and `maxY`, which stay. A plain `{ min_x, ... }` object still converts.
- `Scene.load` fails on a glTF attribute or index accessor the file declares but whose bytes can't be read. It computed normals, used default UVs and colors, or drew without indices instead.
- `Scene.load` fails on a glTF file that reaches one node twice, through a cycle or a node with two parents. glTF requires nodes to form trees.
- `Camera::perspective`, `Camera::orthographic`, `Camera::set_aspect` and `Camera::look_at` return `Result<Camera, CameraError>` in Rust, and Swift calls to the first three now need `try`. A resize handler that hands `set_aspect` a minimized window's width of 0 gets an error and keeps the old aspect. `SceneLoadError` has a new `Camera { index, source }` variant.
- `Scene.load` fails on a glTF primitive drawn as points or lines, on an attribute whose element count differs from `POSITION`'s, and on an index past the last vertex. It drew points and lines as triangles, filled the missing elements with defaults, and sent the bad index to the GPU.
- `Light.set_range` refuses NaN with `LightError::NegativeRange`, the error a negative number gets. NaN passed the check and lit as an unlimited range.
- `Scene.load` fails on a `KHR_lights_punctual` point or spot light with a negative `range`, returning `SceneLoadError::Invalid` with the `set_range` message and the light's index. It clamped the range to 0, which means unlimited. A directional light has no range, so the loader still ignores one there.
- Texture usage masks take the five `TextureUsage` flags only. `create_texture` and `create_storage_texture` refuse any other bit, and `create_storage_texture` refuses an empty mask, with `TextureError::InvalidInput`. wgpu-only bits reached wgpu as a validation error, and unknown ones were dropped.
- `create_storage_texture` and `create_texture` check the usage and dimension the device allows for the format, so a storage texture in `Depth32Float`, `Rgba8UnormSrgb` or `Bgra8Unorm`, and a `Depth32Float` texture with a depth past 1, return `TextureError::InvalidInput`. A format that needs a missing wgpu feature returns `TextureError::UnsupportedFormatForUsage` for any usage, where only `TEXTURE_BINDING` checked it, and a KTX2 file in a compressed format the device lacks returns the same variant, where it returned `TextureError::CreateTextureError`.
- Raw bytes into a depth texture, from `create_texture`, a `create_storage_texture` seed or `Texture.write`, return `TextureError::InvalidInput`. Draw into a depth texture instead.
- `Texture.write`, `write_region`, `get_image` and `read_texture` on a block-compressed texture from KTX2 return `TextureError::InvalidInput`.
- A decoded image with an integer format override, such as `Rgba32Uint`, returns `TextureError::InvalidInput`. Pass raw bytes with a size for an integer texture. An HDR file infers `Rgba16Float`, where `Rgba32Float` never uploaded.
- Raw bytes must match their region exactly. `create_texture` with a size, the `create_storage_texture` seed and `Mipmap.build` refused only short data, and a longer buffer uploaded its first rows. A storage seed of the wrong length returns `RendererError::TextureError(TextureError::InvalidInput)`, where a short one returned `RendererError::CreateTextureError`, and `create_texture` with raw bytes of the wrong length returns `TextureError::InvalidInput`, where short ones returned `TextureError::CreateTextureError`. `Texture.write` and `write_region` without a stride take tight or 256-byte padded rows, and any other length returns `TextureError::InvalidInput`, which Python raises as `InvalidInputError`. `Mipmap.build` refuses a size with a depth past 1.
- A partial write into a texture with a mip chain returns `TextureError::InvalidInput`, since it would leave the smaller levels stale. Create a texture you update in parts with `mipmaps: false`. That includes raw `Rgba16Float` and `Rgba32Float` data, such as a height field or a lookup table, which now gets a CPU mip chain by default where it had one level.
- A size with a path, a URL, a KTX2 file or a `Mipmap` returns `TextureError::InvalidInput` from `create_texture` and `Mipmap.build`, before any file read or fetch. A path dropped the size, and a URL uploaded its encoded bytes as pixels.
- A region with a width, height or depth of 0 is a `TextureError::InvalidInput` error in `write_region`, for arrays, objects, dicts, a `TextureRegion` and the Swift and Kotlin `TextureRegion` record, where a 0 stretched the region to the edge of the texture. Only a missing region means the whole texture: call `write`, or pass `undefined` in JavaScript and `None` in Python. Rust's `TextureRegion` no longer implements `Default`, whose zeros were that sentinel.
- A JavaScript `writeRegion` object and a Python `write_region` dict need their `width` and `height`, or their `maxX` and `maxY`, each max past its min, and one form at a time. Reversed bounds wrote to the edge of the texture, and misspelled keys wrote the whole texture.
- A JavaScript `ImageData` or canvas takes the size it has and an 8-bit RGBA or BGRA format, and a NumPy array takes its own size. A different `size`, or another format, throws or raises `InvalidInputError`.
- A positive `rotation` in `uv_transform`, `texture_transform` and a glTF `KHR_texture_transform` turns the UVs counter-clockwise in texture space, where `v` points down, as the extension's example and the Khronos sample renderer do. The PBR shader turned them the other way, so every rotated texture turned the wrong way.
- `Scene.load` fails on a texture that reads `TEXCOORD_2` or above with `SceneLoadError::Invalid`, which names the slot, such as `occlusionTexture`. The PBR material samples two UV sets, and the loader drew such a texture with `UV0`.
- The PBR material's `material.uv_offset`, `material.uv_scale` and `material.uv_rotation` uniforms are gone. Code that sets them through the material's shader sets `material.uv[i].offset`, `.scale` and `.rotation` instead, where `i` counts the slots in `TextureSlot` order, and `material.uv[i].uv_set` holds the UV set of slot `i`.
- The PBR shader's `lights` is a read-only storage `Lights` at group 0 binding 1, and `light_grid` takes binding 2. It was a uniform `LightArray` of 32 lights. A custom shader that copied the 0.12.6 `LightArray` keeps drawing with the lights that reach it, up to its own array length; that shape goes in 0.13.
- `shader.set` and `shader.get` on `lights.count` and `lights.lights[i]` return `ShaderError::UniformNotFound` in every language, since the renderer fills the lights at each render. `lights.ambient` still sets the ambient term, so `Scene.ambient` works as before. To draw with ambient light alone, where `lights.count = 0` did that, add a black light.
- `list_uniforms` and `list_keys` leave out the `lights` and `light_grid` roots of a shader that declares them.
- The 32-light cap is gone, and so is the warning that dropped a light added before its shader joined the pass. `PassError::LightCapReached { cap }` names the cap of the device, with a new message, and it also fires when a `Model` whose shader would take too many lights joins a pass, and from `render`, before it takes a frame, for lights added before the renderer opened a device. Only shaders that declare `lights` take lights, so a backdrop shader in the pass no longer counts against the cap.
- `Pass::add_shader` returns `Result<(), PassError>`, as the other `Pass` setters that can fail do. A compute shader on a pass of render shaders, or the reverse, fails with the new `PassError::ShaderKind`, which names both kinds, where the pass logged a warning and dropped the shader. A shader that declares `lights` takes every light the pass holds, which 0.12.6 left to `Model`s alone, and fails with `PassError::LightCapReached` when they would take it past the cap. Either error leaves the pass as it was. JavaScript throws an `Error`, Python raises `FragmentColorError`, Swift calls need `try`, and Kotlin throws `FragmentColorException`. `pass.add(&model)` fails with `PassError::ShaderKind` too when the Material's shader doesn't match the pass.
- In a shader with a vertex or fragment entry point, `lights` and `light_grid` name the light roots. A storage `lights` that isn't the read-only `Lights` struct the `Material::custom` page lists, or a storage `light_grid` next to it that isn't `array<u32>`, fails `Shader::new` with `ShaderError::TypeMismatch`, which spells out the layout. 0.12.6 built a fixed-size storage `lights` of any layout as an ordinary root.
- A uniform `lights` that differs from the 0.12.6 `LightArray` in a member name, type or offset takes no lights. It stays an ordinary uniform that `set` reaches, and `Shader::new` logs one warning. In 0.12.6 such a shader took lights through `set` whenever its member paths matched.
- `RendererError` has a new `PassError` variant, which `render` returns for a shader past the light cap. The enum isn't `#[non_exhaustive]`, so a `match` that lists every variant needs a new arm.
- `render` returns an error for a pass the GPU would refuse, before the pass records: a depth target of another size (`RendererError::DepthSizeMismatch`), a target from another `Renderer` (`RendererError::ForeignTarget`), a texture both drawn into and bound, or read and written by one compute dispatch (`PassError::TextureConflict`), a viewport or compute dispatch past the device limit (`PassError::ViewportLimit`, `PassError::DispatchLimit`), and a shader output or blend the target's format can't take (`PassError::InvalidColorTarget`), a blend from an output without alpha included. In 0.12.6 `render` returned success for each, the GPU's error reached only the log, and a target from another `Renderer` panicked.
- A depth target that is no longer registered, or that another `Renderer` created, makes `render` return `TextureNotFoundError`, where the pass drew without depth or with an unrelated texture. So does a PBR map set to such a texture, where it drew the 1x1 default.
- A compute dispatch of 0 in any dimension dispatches nothing, and the pass's storage keeps what it held. A 0 became 1 and ran one workgroup, so pass 1 where that was the intent.
- A viewport side past the device's largest 2D texture is an error from `render`, where the side shrank to the limit. Within the limit a viewport keeps its size, past the target too.
- `Pass.set_target` and `Pass.set_depth_target` refuse a texture with more than one mip level or layer. Create a texture you draw into with `mipmaps: false`.
- `RendererError` has new `DepthSizeMismatch` and `ForeignTarget` variants, and `PassError` new `DispatchLimit`, `TextureConflict` and `ViewportLimit` variants. Neither enum is `#[non_exhaustive]`, so a `match` that lists every variant needs new arms.
- A KTX2 file with more mip levels than its size holds fails to load.
- A `Light` stops lighting once nothing holds it. A `Pass` and a `Scene` hold the lights added to them, so `scene.add(&light)` keeps lighting after the handle drops, but a shader no longer keeps a light it was given after its passes, scenes and handles are gone.
- A `Material::custom` shader that declared `uv_offset`, `uv_scale` and `uv_rotation` in its `material` struct no longer gets them from `uv_transform`. `uv_transform`, `texture_transform` and `texture_uv` write `material.uv`, an array of five `TextureUv` entries whose declaration the `Material::custom` docs show, and log a warning for a shader that lacks it.

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
