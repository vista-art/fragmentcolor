# Changelog

## 0.12.6: Binding and renderer fixes

A patch release with fixes for the language bindings and the renderer, and the calls that let an example read the same in all five languages. The fixes come first, then the calls each binding gained to match the others, then the changes that can break existing code.

### Fixes

`Pass.setDepthTarget(texture)` on Swift and Kotlin now passes the depth `Texture` to Rust as a `Texture`. The extensions handed over the texture's handle where a `TextureTarget` handle was expected, so Rust read the texture's memory as the wrong type, which is undefined behavior. They now call a new `setDepthTexture` binding that takes the `Texture` directly.

On Android, `renderer.createTarget(surface)` builds the target on the renderer you call it on. The JNI path created a new `Renderer` for every surface, so the target lived on a different GPU device than the renderer drawing into it, and `render` either failed with `NoContext` or mixed the two devices. The JNI helper now only turns the `Surface` into an `ANativeWindow` pointer, and the Kotlin extension passes that pointer to the calling renderer's `createTarget`.

`Model(mesh, material)` and `model.material()` no longer reset the Material on Swift, Kotlin, JavaScript, and Python. Both rebuilt the Material with `Material::custom`, which put the caller's Material back to the `Opaque` alpha mode and single-sided culling but left the shader's alpha flag untouched, so `Mask` and `Blend` materials drew with mismatched pipeline state. The bindings now share the Material the way Rust does.

Python's `Renderer.create_target` keeps one target per renderer. It cached the target on the canvas and handed it to any renderer that asked, even one on another device.

`Shader.set` on the web checks what it receives before binding a texture. Any object exported by the library used to pass as a `Texture`, so handing it a `Shader`, `Pass`, or `Mesh`, or a plain object with a copied `__wbg_ptr` field, read that object's memory as a texture. A `Texture` binds as before. Anything else throws a type mismatch error, as it does in Python.

`Shader.set` follows one set of rules in every language. Each binding used to coerce values on its own, so a call that worked in Python could fail in JavaScript. Rust now shapes the value to the type the shader declares for the key, nested and indexed keys included, and the bindings only pass it along. A number fits an `f32`, `i32`, or `u32` uniform, as long as it is a whole number in range for the integer types. A vector takes exactly as many numbers as it has components, and `mat2x2`, `mat3x3`, and `mat4x4` take 4, 9, or 16. Anything else throws an error that names the key and both sides, such as `resolution: expected vec2<f32>, got 3 number(s)`. The call itself reads the same in all five languages.

The shared rules fix these bugs:

- On the web, plain numbers and arrays reach `u32`, `i32`, and integer vector uniforms. Arrays were always read as floats, so `shader.set("count", 3)` failed on a `u32`.
- `mat2x2` uniforms work from JavaScript, Swift, and Kotlin, which read four numbers as a `vec4`.
- Python raises on a list of the wrong length. A three-item list for a `vec2` used to succeed without writing anything.
- Indexed keys such as `buf.items[2].v` check the declared type. They compared only byte counts, so a `vec4<u32>` landed in a `vec4<f32>` slot.
- JavaScript and Python can send raw bytes to a storage-buffer or push-constant root: a `Uint8Array` on the web, or `bytes`, `bytearray`, and `memoryview` in Python. Swift takes `Data` and Kotlin takes `ByteArray`.
- `Shader.get` reads the current value. Struct roots returned the defaults reflected from the shader, whatever had been set since. Struct, array, and buffer roots now read as their current bytes, and indexed keys can be read too.

A few inputs that used to fail now convert. Any typed array other than `Uint8Array` counts as numbers, so an `Int32Array` set into a `vec2<f32>` becomes floats. In Rust, an integer literal fits a float uniform, so `shader.set("scale", 2)` works on an `f32`. A bool writes 1 or 0 into a numeric uniform.

On Swift and Kotlin, the generated `set(key:value:)` takes a `UniformValue` (a bool, a number, a list of numbers, a texture, or bytes) in place of a `UniformData`. The `shader.set("key", value)` overloads that the docs use read the same. Swift adds `Int`, `Double`, `[Int]`, and `Data` overloads, and Kotlin adds `DoubleArray` and `ByteArray`.

`scene.setPasses([a, b])` on the web leaves `a` and `b` usable. The binding took the passes by value, so wasm-bindgen moved each one out of its JavaScript handle, and any later call on `a` threw `Attempt to use a moved value`. It now copies each handle, as Python, Swift, and Kotlin do.

`renderer.render([a, b], target)` on the web records one frame. It rendered each item as a frame of its own, so on a canvas the last item's frame replaced the others. It now collects every item's passes in order and records them once, as Rust and Python do. Each item still brings its own pass graph, and which dependencies a list should pull in is open for a later release. `renderer.load([a, b])` loads every item, where it threw before.

JavaScript indices and counts must be whole numbers. `scene.addTo(-1, model)` added the model to pass 0, because the index went through a saturating float cast, and `getPass` and `Mesh.setInstanceCount` let the wasm ABI turn -1 into 4294967295. `Scene.addTo`, `Scene.getPass`, `Mesh.setInstanceCount`, and `Mesh.setIndices` now throw on a negative, fractional, or NaN value. `setIndices` also takes any numeric typed array, such as a `Uint16Array`.

`Pass.require` links each dependency once on Swift and Kotlin, and for a list on the web. A list of renderables gave `require` every pass its items require, so the documented example, where `compose` requires `color` and then `blur_y`, failed with `DuplicateDependency: color` on mobile. `require` now links the items themselves.

Python renders the passes a `Pass` requires. A Python `Pass` handed `render` only itself, so the passes linked with `require` never ran. A single `Pass`, `Shader`, `Mesh`, or `Scene` now renders as it does in Rust. A list still renders its items in order, without the passes they require, as a Rust list does. `require` raises `TypeError` for a `str` or a list item that isn't a renderable, where a `str` read as a list of letters.

Kotlin creates a texture from raw pixels and their size. The generated Kotlin for `renderer.create_texture((pixels, [w, h]))` dropped the size, so the bytes went to the image decoder and the call failed at run time. The Kotlin extensions now carry overloads Swift already has, and the generated examples call them as the Rust examples read: `renderer.createTexture(pixels, listOf(w, h))`, `createTexture(path)` and `createTexture(chain)`, `Shader(listOf(...))` for a shader built from parts, `TextureRegion.from(listOf(...)).withStride(...).withRows(...)`, and `texture.writeRegion(bytes, listOf(x, y, w, h))`. Kotlin examples also keep an integer array as integers, where they wrote floats: a byte list holds `Int`s, and a size or a region holds unsigned literals, such as `listOf(64u, 64u)`.

Swift `texture.writeRegion(bytes, [UInt32])` takes the 2, 4, and 6 number forms that JavaScript, Python, and Kotlin take. It wrote only `[x, y, w, h]` and ignored any other array without an error, and now throws for one. `writeRegion` takes a `ScreenRegion` on both platforms, and Kotlin gains `texture.write(List<Int>)`. Swift `TextureRegion.from` stops with a message on an array it can't read, where it returned a whole-texture region, and lists of `Int` bytes throw for values outside 0 to 255, which `write` clamped and `createTexture` trapped on.

Python reads a NumPy array passed as texture data in row-major order, whatever its memory layout. `Texture.write`, `Texture.write_region`, `Mipmap.build`, and `Renderer.create_texture` copied the array's memory as it lay, so a Fortran-ordered array uploaded its pixels in the wrong order, and a flipped or sliced view raised `ndarray must be contiguous`. Each now reads the array the way NumPy indexes it. Where a method takes raw bytes, a `uint8` array is tried before `bytes` and int sequences, so a one-dimensional array is copied in one step instead of element by element.

The Python `VertexValue` factory keeps the type it builds. `VertexValue.u32(3)` returned a plain Python `3`, which `Vertex.set` and `Instance.set` then read as a float, so no Python vertex or instance attribute could feed a `u32`, `i32`, or integer vector shader input. The factory returns a `VertexValue` now, and `set` keeps its type. Plain numbers and lists still read as floats, as they do in JavaScript.

Passes rendered together keep their own uniforms. `Renderer::render` records every pass of one call into a single submission, and uniform uploads reach the GPU at submit. The renderer rewound its uniform pool at the start of each pass, so every pass got the same offsets and drew with the values of the last pass. In the `multipass_shadows` example the shadow pass drew the main circle, and a compute pass followed by a render pass read the render pass's uniforms. The pool now rewinds once per `render` call.

Shaders that share a source get one pipeline per vertex layout. The pipeline cache keyed on the shader and the target state, but the vertex layout came from whichever mesh built the pipeline first. A second shader with the same source, drawing a mesh with a different stride, attribute offsets, or per-instance attributes, reused that pipeline and drew the wrong vertices or nothing at all. The layout is part of the key now, and building it no longer leaks memory on every frame.

Storage buffers upload only when they change. Every render copied the bytes of each storage buffer into a new allocation and threw them away unless the GPU buffer was new, which came to about 256 MB per frame at eight million particles. The renderer now reads them only to create or resize the buffer, or after `set` changes them, and the `particles` example went from 43 fps to the display's 60 at that size.

Shaders can use atomics in storage buffers. Reflection rejected `atomic<u32>` and `atomic<i32>` with "Unsupported type", so a shader that counts with `atomicAdd` failed to build. An atomic reflects as its scalar, and another shader can read the same buffer by name through a plain `array<u32, N>` view.

A readback that times out or fails no longer breaks the ones after it. `get_image` and `read_storage` wait up to five seconds for the GPU. On a timeout they left their pooled buffer mapped, and every later readback of the same size failed with `BufferStillMapped`. The failed buffer now leaves the pool, and the next readback gets a fresh one.

A render made only of compute passes leaves the target alone. `Renderer::render` acquired a frame even when no pass drew into it and presented that frame untouched, so compute work failed whenever the target had no frame to give, as with a minimized window. Compute-only calls now skip the frame.

Compute passes reuse their pipelines. The compute pipeline cache keyed on the memory addresses of bind group layouts that every render built from scratch, so the key never matched. Each compute pass compiled a new shader module and pipeline on every frame and kept all of them, which grew memory for as long as the app ran. The key now holds the layout entries themselves, and layouts are built only along with a new pipeline.

`Texture.set_sampler_options` changes how the texture is sampled. The renderer kept each texture's sampler but never read it, and bound a new default sampler to every `sampler` binding, so wrapping and filtering options never reached the shader. A `sampler` now uses the options of the texture in its bind group. When a group holds several textures, each sampler takes the one with the lowest binding between the previous sampler and itself, so the PBR material's shared sampler follows its base color map. A `sampler_comparison` compares with `LessEqual` when the texture sets no `compare`, and a plain `sampler` next to a depth texture does not compare. A sampler with no texture in its group uses the default options, so it filters linearly where it used to pick the nearest texel.

`mat3x3<f32>` values reach the GPU in the layout WGSL declares. WGSL pads each `vec3` column of a `mat3x3` to 16 bytes, so the matrix spans 48 bytes, but the renderer packed its 9 values into 36. The second and third columns landed at the wrong offsets in uniforms, struct fields, and storage buffers alike, so shaders read a shifted matrix. The renderer now writes the padding, and `Shader.get` still returns the 9 values.

`TextureOptions.sampler` takes effect when a texture is created. `Renderer.create_texture` and `Renderer.create_storage_texture` built every texture with the default sampler and dropped the options they were given, so a texture wrapped and filtered as asked only after a call to `Texture.set_sampler_options`. Textures loaded from glTF use the wrap and filter settings of their glTF sampler. glTF repeats by default, so UVs outside 0 to 1 now tile the texture where they used to clamp to its edge.

`Texture.write` and `Texture.write_region` take tightly packed rows at any width. Without an explicit `bytes_per_row`, the write path padded each row to a multiple of 256 bytes and asked for that much data, so a texture taller than one row failed with `Input data smaller than required for layout` whenever its row was not already a multiple of 256 bytes. A 2x2 RGBA texture asked for 264 bytes instead of 16. Only copies between a buffer and a texture need 256-byte rows, and `Queue::write_texture` takes any stride. A row now defaults to the region width times the bytes per pixel. Data padded the way the old default asked still uploads intact: when its length is exactly that of rows padded to 256 bytes, it reads at the padded stride. An explicit `bytes_per_row` only has to cover the row width, so a tight stride such as 16 is accepted too. The fix reaches every binding.

A shader with two `@vertex` or two `@fragment` functions draws with the first of each. Render pipelines took the last entry point of each stage, while vertex input reflection and compute pipelines took the first, so a module with two vertex entry points matched its meshes against one and drew with the other. All three now pick the same entry point.

A mesh whose instance list shrinks draws only the instances it holds. The renderer counted instances when it created the instance buffer and kept that buffer for any list that fit. After `clear_instances` and a shorter `add_instances`, the mesh kept drawing the old count, and the extra instances read stale rows past the new end. The count now follows every upload.

`Renderer.create_target` returns an `InitializationError` when the GPU adapter cannot present to the surface. Such a surface lists no formats. That happens with a second canvas on WebGL2, where each canvas needs its own `Renderer`. `create_target` configured the surface anyway with a fallback format, so the call succeeded and wgpu's `UnsupportedQueueFamily` error only reached the log. Every binding reports it as an error the caller can catch, and its message names the cause.

`Renderer.create_texture` honors `TextureOptions.usage`. Both upload paths set `TEXTURE_BINDING | COPY_DST` and ignored the option, so no texture made by `create_texture` could be read back, and `read_texture` failed on it in every language. The usage bits now join the two the upload needs, so `TextureUsage::COPY_SRC` makes the texture readable. Python takes `usage=` and the keyword `sampler=` on `create_texture`, and `usage=` on `create_storage_texture`, where `usage_bits=` still works with a `DeprecationWarning`.

A mesh whose vertices or instances disagree on an attribute fails to draw with an error. The mesh layout comes from its first vertex and first instance, and packing wrote each later value as it was. A key with another format or length, such as a three-number `uv` after a two-number one, shifted every attribute that followed it, and a position of another dimension was zero-filled. Rendering now raises `MeshError::InvalidVertex`, `InvalidInstance`, or `LayoutMismatch`, the binding's error in other languages.

`Scene.load` reads glTF files whose meshes carry application attributes such as `_ID`. glTF lets an application name its own vertex attributes with a leading underscore, but the parser rejected them unless the `gltf` crate's `extras` feature was on, so these valid files failed to load. The feature is on now and adds no dependency. The loader still skips application attributes and `extras` objects.

The committed Swift wrapper in `platforms/swift/Sources/FragmentColor/generated/` is regenerated. Swift Package Manager compiles it from the release tag, and it predated 0.12.0, so it lacked `Scene`, `Model`, `Material`, `Camera`, `Light`, and the rest of the 0.12 surface that the Swift extensions build on. The iOS healthcheck now fails when the committed wrapper differs from what `./fc build_ios` generates.

Each release tag pins its own xcframework. Swift Package Manager builds the package from the tag, but the tagged `Package.swift` pointed at the previous release's binary because the pin reached `main` after the tag. An app on `from: "0.12.5"` got the 0.12.4 xcframework next to the 0.12.5 Swift wrapper. A new `Prepare release` workflow builds the xcframework, commits its checksum, and drafts the release on that commit. Publishing checks the attached binary against the pin rather than building a fresh one. Tags up to 0.12.5 keep their old pins, so move Swift dependencies to 0.12.6.

The npm package no longer ships the test harness. `wasm-bindgen-test` was a runtime dependency of the web build, so the package exported a `WasmBindgenTestContext` class and the `__wbgtest_*` helpers next to the real API. It is a development dependency now, and the bundle carries only the library's own exports.

Unused build inputs are gone. The web build no longer declares `serde-wasm-bindgen` or `gloo-utils`, which no code called, and the stale `generated/wasm_exports.txt` list, which nothing read, is deleted. The build stops writing `generated/api_objects.txt` too. Its only reader was a step that branded the JavaScript classes, and `impl_fc_kind!` replaced that step. The Python extension builds without pyo3's `experimental-async` feature, since no Python method is async.

The Python package declares NumPy as a dependency. `TextureTarget.get_image()` returns a NumPy array, but the wheel did not require NumPy, so on an environment without it the call failed with a panic from the NumPy bridge instead of returning pixels. `pip install fragmentcolor` now pulls in `numpy>=1.16`.

The Python type stub tracks the API again. `fragmentcolor.pyi` ships in the wheel and feeds editor hovers and type checkers, but it had been frozen since May and lacked `Scene`, `Model`, `Material`, `Camera`, `Light`, and `Mipmap`. The build regenerates it on every run. Signatures come from the Rust sources: a binding's own parameter types, or, for a parameter pyo3 takes as an untyped object, the Rust method the binding documents, followed through the crate's conversions. A new method, parameter, or accepted type shows up without a manual edit. Each docstring carries the method's text from the API docs and the same Python example the website shows.

FragmentColor builds from read-only sources, so docs.rs can document it again. Every build ran the docs pipeline, which writes the generated examples, the website pages, the Python stub, and the embedded shader lookup into the crate's own folder. docs.rs mounts that folder read-only, so the build script panicked on its first write, and no release from 0.10.7 to 0.12.5 has documentation there. A build from crates.io or from a vendored copy wrote into that copy too. The pipeline now runs only in a repository checkout, where `.git` sits next to `Cargo.toml` and `DOCS_RS` is unset, and the embedded shader lookup goes to `OUT_DIR`. The published crate carries only the files a build reads, which shrinks it from 6.9 MB to 0.56 MB and the Python source distribution from 7.1 MB to 0.63 MB.

The generators that turn the API docs into JavaScript, Python, Swift, and Kotlin examples no longer keep hand-written lists of API facts. They read which methods throw or are async, which arguments are floats, unsigned ints, or lists, which enum entries Kotlin spells in capitals, and what each binding exports from the Rust sources and the hand-written Swift and Kotlin extensions. Stale entries went with the lists. Generated Swift no longer puts `try` before `setIndices` or `removeMesh`, or `await` before `Scene.load`, and Python examples import `VertexValue` when the Rust example does. The `Pass.from_shader` examples call `from_shader` in every language instead of building the pass by hand. Swift adds an unlabeled `Pass.fromShader(name, shader)` overload for them, next to the generated `fromShader(name:shader:)`.

Links to API methods open the method on its object's page. The docs linked a method as `fragmentcolor.org/api/<object>/<method>`, a path the site doesn't serve, so about 120 links in the website, the Rust hover docs, and the Python stub led to a 404. They now point at the object's page and the method's heading, such as `/api/core/renderer#rendererload`. The build fails on a link that names a page or heading the site doesn't build, and its error gives the link to use.

Swift and Kotlin examples keep the code spans in their comments. The transpilers turned every backtick into `"""`, so a comment such as "uploaded lazily by `Renderer.load`" read `"""Renderer.load"""` on the Swift and Kotlin tabs. Only the backticks that open and close a string become `"""` now.

The generated examples had two more defects. The Swift and Kotlin transpilers stripped the semicolons from the WGSL inside their shader strings, so the shaders in those examples compiled but could not parse as WGSL at run time. The Kotlin transpiler read a trailing comma in a pixel array as one more value, which gave the 2x2 textures in the renderer load example and the five material texture examples 17 bytes instead of 16. Both now match the Rust source.

The build deletes generated examples whose API page is gone. It writes a JavaScript, Python, Swift, and Kotlin example for each page in `docs/api`, but a deleted or renamed page left its examples behind under `platforms/`, and two of the Python ones did not even parse. Each build now removes the example files it did not write, the way it already pruned the website pages. That cleared out 84 files from pages such as `Pass.add_target` and `Shader.duplicate`.

The `TextureTarget` page lists `get_image`, `resize`, and `size` again, and `Shader.default` has an entry of its own. The export took each object's methods from the API map, which leaves out trait methods, so these four pages were missing from the site and the healthchecks never ran their examples. It now renders every method page in the object's docs folder.

The Rust examples for `TextureTarget.size` and `TextureTarget.resize` compile, and the API pages print each generic type once. Only the Python, web, and mobile wrappers carry the two `TextureTarget` pages, so no Rust build checked their examples, which called the `Target` methods without importing the trait, compared a `Size` with an array, and compared the `Option<u32>` depth with `1`. The `size` heading now gives `depth` its `Option<u32>` type. The website export wraps a generic type such as `Vec<u8>` in a code span so MDX does not read it as a tag, but it also left the type's name in front of the span, so the `Target.get_image` heading read `Vec` twice and the `Pass`, `Shader`, and `Quad` pages repeated `Result` and `vec2`.

`Shader::default` shows its documentation in Rust editors, as `Shader.default` already did in the other bindings. The build only checked the methods marked `pub`, which leaves out trait methods, so the page never reached the Rust method. The build now checks `Default::default` and the `Target` methods too. The `TextureTarget` and `WindowTarget` pages list `size`, `resize`, and `get_image` in the order the `Target` trait declares them.

The generated JavaScript examples write Rust's `None` as `undefined`, which wasm-bindgen returns, and build a `ScreenRegion` with `new ScreenRegion(origin, size)` where they passed a bare array. Python examples spread a tuple over a static method's parameters as they already did for instance methods, so `Mipmap::build((bytes, format, [8, 8]))` reads `Mipmap.build(bytes, format, size=[8, 8])`.

The API docs gain pages for `Instance`, `PassInput`, and `VertexValue`, which every binding exposed, and `Mipmap::level` gets a page of its own. Several pages now say what the code does: where `Shader::new` and `Shader::fetch` find registry slugs, URLs, and paths on each platform, that only a depth `Texture` works as a depth target, that a pass viewport isn't applied yet, and that adding a `Light` can fail at the 32-light cap, depending on the order. The window target pages build a real window target, and the `Renderer::create_texture` and `Mipmap` examples read real image bytes in every language.

### Parity across languages

Every binding names its types as Rust does. The window target is `WindowTarget`, where JavaScript's `renderer.createTarget(canvas)` returned a `CanvasTarget`, Python's a `RenderCanvasTarget`, and Swift and Kotlin's a `MobileWindowTarget`. On Swift and Kotlin, `renderer.createTextureTarget` returns a `TextureTarget` where it returned `MobileTextureTarget`, `Pass.getInput` returns a `PassInput` instead of a `MobilePassInput`, and the texture region record is `TextureRegion`. The old names still work. The JavaScript package exports the same class under both names, in the code and in the typings, Python keeps `RenderCanvasTarget` importable as an alias, and Swift and Kotlin keep deprecated typealiases, so code that uses them compiles with a warning. The empty Swift and Kotlin `WindowTarget` and `TextureTarget` classes that no method returned are gone. On the web, `WindowTarget.getImage()` returns a Promise that resolves to an empty `Uint8Array`, as it does in every other binding, where it threw.

Swift and Kotlin render a pass into a `Texture`. `pass.setTarget(texture)` took only a `TextureTarget` on mobile, while Rust, JavaScript, and Python also take a `Texture`. The extensions call a new generated `Pass.setTargetTexture`, the way `setDepthTarget(texture)` calls `setDepthTexture`.

Texture and window targets resize to a `Size` on Swift and Kotlin, so `target.resize(other.size())` works, and Kotlin takes a `[w, h]` list.

`Pass.require` takes a `Scene` in every binding. On the web it returned without doing anything for a value it didn't recognize, a `Scene` included, and skipped unrecognized items in an array. It now throws on anything else. Swift and Kotlin accepted a `Scene` only as a raw `RenderableHandle`. On the web, `Pass.setComputeDispatch` throws on a negative, fractional, or missing size, which the wasm ABI used to wrap or truncate.

`Pass.add` and the `Scene` builders return their receiver in every binding, as in Rust, so `scene.add(model).add(camera)` and `pass.add(model).add(camera)` chain. That covers `Scene.add`, `addTo`, `addPass`, `ambient`, `noDefaults`, `noDefaultCamera`, `noDefaultLight`, `setDefaultCamera`, and `setDefaultLight`, in snake_case in Python. They returned nothing in JavaScript, Python, Swift, and Kotlin. On the web each returns a second handle on the same object. Code that ignores the result works as before, and Swift warns about an unused result only for the generated methods with no extension overload, such as `noDefaults()`.

`Pass::set_clear_color` takes any `Color`, so `[r, g, b]` clears to an opaque color, and Python `Pass.set_clear_color` takes 3 or 4 numbers. The `PassInput` that `get_input` returns is readable in every binding: `loads_previous()` and `color()` in Rust and Python, and `loadsPrevious()` and `color()` in JavaScript, Swift, and Kotlin.

`ScreenRegion` reads the same in every binding. JavaScript gains `new ScreenRegion(origin, size)`, `minX`, `minY`, `maxX`, and `maxY`, and `width()` and `height()`, and takes a `ScreenRegion` wherever it takes a region. Python exposes `min_x`, `min_y`, `max_x`, and `max_y` and the two methods, and compares and hashes regions by value. Python `Pass.set_viewport` takes `[x, y, w, h]` and `(w, h)` next to a `ScreenRegion`.

Swift and Kotlin read a viewport the way Rust does. `pass.setViewport([x, y, w, h])` covers `w` by `h` pixels from `(x, y)`, where Swift read the four numbers as two corners, and `[w, h]` covers `w` by `h` from the origin. Kotlin gains the list form. `ScreenRegion(origin, size)` builds a region on both platforms, as `ScreenRegion::new(origin, size)` does in Rust, and the generated examples call it. The renderer doesn't apply a pass viewport yet, so no output changes.

Matrices take both shapes. `Shader::set` takes a flat matrix in Rust: 9 numbers for a `mat3x3` and 16 for a `mat4x4`, as an array or a `Vec<f32>`, column after column, as the bindings take them. `Shader::get` reads them back, reads a `mat2x2` as `[f32; 4]`, and reads `Vec<f32>`, `Vec<u8>`, and the `TextureId` bound to a texture key. Python `Shader.set` takes a nested list or a 2-D array, one column per inner list, so `shader.set("view_proj", camera.view_proj())` works, and on the web `Shader.set` flattens one level of nesting, so a `mat3x3` written as three columns sets. `Model.setTransform` on the web and Python `Model.set_transform` take four columns of four as well as a flat list of 16. On the web, `Shader.set` also reads an `ArrayBuffer` or a `DataView` as raw bytes, as it already read a `Uint8Array`.

`Shader.set` takes the numbers each platform writes naturally. Kotlin adds `List<Float>`, `List<Double>`, `List<Int>`, `Long`, and `LongArray`, so `shader.set("view_proj", camera.viewProj())` compiles. Swift adds `UInt`, `[UInt]`, `Int64`, and `[Int64]`. Both take a matrix as a list of its columns, the shape of Rust's `[[f32; 4]; 4]`. The `float`, `floats`, and `bytes` accessors read the `UniformData` that `Shader.get` returns as a number, a list of numbers, or bytes.

`Shader.set_registry` works on native builds. A slug, or a URL under the active registry, resolved from the shader library compiled into the binary, so the registry was never asked for a shader the binary carries. After `set_registry` the registry answers first, and the embedded copy stands in when it can't be reached, as in a build without the `network` feature. The default registry still resolves offline from the binary. A full URL into the default registry, such as `https://fragmentcolor.org/shaders/sdf2d/circle.wgsl`, gets the same fallback. After `set_registry` it counted as an ordinary URL, so builds without `network`, Swift and Kotlin among them, failed with a network error. It is now fetched as written, with the embedded copy behind it.

Swift builds a shader from parts with `Shader([...])`, as JavaScript, Python, and Kotlin do. `Shader.new([...])` still works and is deprecated, and so is Kotlin's top-level `ShaderFetch`, which `Shader.fetch` already covers.

`renderer.createTexture` on the web reads every option it documents. It ignored the `sampler` option its own doc listed, dropped an option of the wrong type, and had no `usage` option. It now reads `{ size, format, mipmaps, sampler, usage }`. `sampler` takes a `SamplerOptions` or a plain `{ repeatX, repeatY, smooth, compare }` object, `usage` takes a `TextureUsage` or its bits, and an option of the wrong type throws. An `ImageData`, `HTMLCanvasElement`, or `OffscreenCanvas` source brings its own width and height, so `createTexture(imageData)` needs no size. A `Texture` is accepted as a source. `createStorageTexture` takes a `TextureUsage` as well as a number, and its seed data as a `Uint8Array`, an `ArrayBuffer`, or an array of numbers.

JavaScript and Python create KTX2 textures. Bytes, a path, or a URL whose data starts with the KTX2 identifier load through the KTX2 path, which only the explicit Rust and mobile KTX2 inputs reached before.

Rust's `Renderer::create_texture` takes a string: `create_texture("logo.png")` reads the file, and a string that starts with `http://` or `https://` is a URL, as in Swift and Kotlin. Python reads an http(s) `str` as a URL too, where it was always a path, and takes an `os.PathLike`. Python `create_texture` also takes a `bytearray`, a `memoryview`, a tuple of ints, and an existing `Texture`, which comes back as is. The JavaScript dispatcher takes a `Texture` as well.

Swift and Kotlin create textures with the calls JavaScript and Python use. `createTexture(bytes)` decodes an encoded image, `createTexture(bytes, format)` decodes it into a format, `createTexture(pixels, [w, h])` uploads raw pixels, and `createTexture(bytes, options)` spells out every option. Swift takes the pixels as `Data` too, builds a full mip chain for them as every other binding does, and throws for a size that isn't 2 or 3 numbers, where it decoded the pixels as an image. `createStorageTexture(size, format, data, usage)` takes its arguments in order, with `data` and `usage` optional. `createTextureTarget` and `createDepthTexture` take a `Size` on both platforms and a `[w, h]` list on Kotlin. Swift's one-tuple forms, `createTexture((bytes, size))` and `createStorageTexture((size, format))`, still work and are deprecated.

Every binding names the `TextureUsage` flags `COPY_SRC`, `COPY_DST`, `TEXTURE_BINDING`, `STORAGE_BINDING`, and `RENDER_ATTACHMENT`. JavaScript exposes them as numbers, as WebGPU's `GPUTextureUsage` does, so `TextureUsage.COPY_SRC | TextureUsage.TEXTURE_BINDING` works. Python exposes them as `TextureUsage` values that combine with `|`, and the module exports `TextureUsage`. They combine with `|` on Swift and `or` on Kotlin, where `createTexture` and `createStorageTexture` take a `usage`, so `createTexture(pixels, [w, h], usage: .COPY_SRC)` makes a texture that `readTexture` can read back.

`Renderer::read_texture` and `unregister_texture` take `impl Into<TextureId>`, so Rust passes `texture.id()` or a plain `u64` as the bindings do.

`Mipmap` reads one level or all of them in every binding. Rust gains `Mipmap::level(index)`, which every binding had, and Python, JavaScript, Swift, and Kotlin gain `Mipmap.levels()`, which Rust had. On the web it returns one `Uint8Array` per level, level 0 first. An index past the last level raises `InvalidInputError` in Python. `Mipmap.build(bytes, format)` and `Mipmap.build(bytes, format, size)` take their arguments in order on Swift, and Kotlin takes a `[w, h]` list for the size.

Python value types compare by value. `chain.format() == TextureFormat.Rgba8UnormSrgb` is `True`, and so are equal `Size`, `ScreenRegion`, `SamplerOptions`, and `TextureId` values. Each was compared by identity, so two equal values were never equal. `TextureFormat`, `TextureId`, and `ScreenRegion` hash too, and `TextureId` exposes `.id`. JavaScript gains `new Size(width, height, depth?)` and `new SamplerOptions(repeatX?, repeatY?, smooth?, compare?)`, which made objects without a backing value before. Swift and Kotlin give `SamplerOptions`, `Size.depth`, and every `TextureOptions` field but `format` their Rust defaults.

Python has one `Size` class, and its fields are readable. pyo3 exposed two classes named `Size`. The one that `from fragmentcolor import Size` imports had no attributes, so `texture.size.width` raised `AttributeError`, while `TextureTarget.size` and `RenderCanvasTarget.size()` returned an unregistered copy that did carry `width`, `height`, and `depth`. Every `size` accessor now returns the registered class, and its fields read and write as attributes. Its `depth` follows the Rust value, as in the other bindings, so a window target reports `None` where the copy always said 1. `Size(width, height, depth=None)` builds one, and every method that takes a size accepts it next to the list, tuple, and dict forms.

Python can build `SamplerOptions` and `CompareFunction`. Both appeared in the `Texture.set_sampler_options` signature, but `SamplerOptions` had no constructor and `CompareFunction` was missing from the module, so the method only worked with a dict. `SamplerOptions(repeat_x=True, compare=CompareFunction.LessEqual)` builds the options, a keyword left out keeps the Rust default, and each field reads and writes as an attribute. The dict form still works.

Every binding has the `Vertex.NORMAL`, `TANGENT`, `UV0`, `UV1`, `COLOR0`, and `COLOR1` constants, so the JavaScript, Swift, and Kotlin examples read `Vertex.UV0` like the Rust and Python ones, where they spelled out `"uv0"`. The vertex methods take Rust's `(position, uv)` and `(position, uv, color)` forms: `[position, uv]` and `[position, uv, color]` in the web `Mesh` methods, and `mesh.addVertex(position, uv)` and `mesh.addVertex(position, uv, color)` on Swift and Kotlin. Swift and Kotlin `addInstance` takes a `Vertex` or attributes by name, in the order written, and a position there may hold one number, as in Rust. A `VertexValue` class gives JavaScript the exact-type factories Python has, `VertexValue.f32` through `VertexValue.i32x4`, for integer shader inputs. An integer that doesn't fit its type throws. On the web, `Vertex.set` and `Instance.set` throw on a non-numeric item, where they wrote 0.

Rust takes the vertex and instance forms the bindings take. `Mesh::add_vertex` and friends take a bare `f32`, `[f32; 1]`, and `[f32; 4]` next to 2 and 3 component positions, and an `Instance` builds from a map of named `VertexValue`s, set in key order. `Vertex::set` and `Instance::set` are `#[must_use]`, since each returns the updated value. Python reads a one-item list as a 1-component position.

Kotlin passes positions straight to a mesh, as the other bindings do. A new `MeshExtensions.kt` mirrors the Swift extensions. `addVertex`, `addVertices`, and `Mesh.fromVertices` take positions as `List<Float>`, `List<Double>`, `FloatArray`, or `DoubleArray`. `Vertex`, `Vertex.pbr`, and `Quad` take every float list type. `Vertex.set` and `Instance.set` take `DoubleArray` and `List<Double>`. The generated Kotlin examples drop the `Vertex(...)` around each position.

Swift's vertex API fills its gaps. It gains `Mesh.fromVertices([vertex])`, scalar `Instance.set`, `Double` and `Int` scalars for `Vertex.set`, and an unlabeled `set(key, VertexValue)`. An `Int` stays an `f32`, as untyped numbers are everywhere. `addVertex(vertex)` and `addVertices([vertex])` no longer throw, and `Instance.set` warns when its result goes unused, since it returns a copy. `Vertex.set` and `Instance.set` stop with a message on a `[Float]` of 0 or more than 4 numbers, which wrote a misaligned value.

Python takes a `Mesh` wherever the other bindings do. `Renderer.render`, `Renderer.load`, and `Pass.require` accept a `Mesh` on its own or inside a list, as Rust, JavaScript, Swift, and Kotlin already did. A lone `Mesh` used to raise `TypeError`, and a list holding one raised `AttributeError`. `Mesh.add_vertex`, `Mesh.add_vertices`, and `Mesh.from_vertices` also take the `(position, uv)` and `(position, uv, color)` tuples that Rust's `Into<Vertex>` accepts, next to every shape they took before.

Swift and Kotlin load a scene from the bytes of a `.glb` file, as Rust, JavaScript, and Python do. `Scene.load(data)` takes `Data` on Swift and `Scene.load(bytes)` takes a `ByteArray` on Kotlin, next to the path form. Both call a new generated `Scene.loadBytes(bytes)`, since uniffi can't overload `load`. The `Scene.load` example's Swift and Kotlin tabs now read the file's bytes, where they passed its path.

`Scene.load` skips the file's own cameras or lights in every binding, as `SceneSource::gltf(src).cameras(false)` does in Rust. JavaScript takes a second argument, `{ cameras, lights }`, Python takes `cameras=False` and `lights=False`, and Swift and Kotlin take `Scene.load(source, cameras: false)` and `Scene.load(source, cameras = false)`. Both flags default to `true`. On the web, `Scene.load` also takes an `ArrayBuffer` or any view of one, such as a `DataView`, so `Scene.load(await response.arrayBuffer())` works without wrapping the buffer, and Python takes a `bytearray`.

The five `Material` texture setters take any source `createTexture` takes in every binding, as Rust's `impl Into<TextureInput>` does. They took only a `Texture`. A `Texture` binds at once, and bytes, a path, a URL, an image, or a mip chain uploads when the scene loads. On the web the setters take `createTexture`'s options too. On Swift and Kotlin, `material.baseColorTexture(bytes)`, `(bytes, format)`, `(path)`, and `(chain)` work, and so do the other four setters. uniffi can't overload, so they call new generated methods such as `Material.baseColorSource`.

Kotlin's `Camera`, `Light`, `Material`, and `Model` take `Double`, `FloatArray`, and `DoubleArray`, as Swift's do, so `Camera.perspective(1.047, 1.0, 0.1, 100.0)` compiles. `model.setTransform` also takes a 4x4 matrix as its four columns.

TypeScript accepts plain arrays for the vector parameters of `Model`, `Camera`, `Light`, `Material`, and `Scene`. The typings declared them as `Float32Array`, so a strict project rejected `Light.directional([0, -1, 0], [1, 1, 1])`, which the docs show and the runtime accepts. They are `ArrayLike<number>` now, and `render`, `load`, `require`, `setPasses`, and `Scene.load` declare the types they take.

### Behavior changes

JavaScript methods throw `Error` objects. Most failures reached JavaScript as a bare string, so `e.message` was `undefined`, `e instanceof Error` was false, and the error carried no stack. Every method now throws an `Error` whose `message` holds the old text, as the methods built on `JsError` already did. This is a breaking change for code that compares a caught value to a string or calls string methods on it. Read `e.message` instead.

`new Shader(source)` throws, and `Shader.fetch` rejects, when the source is invalid. On the web both logged the error and returned the default red shader, so a typo in the WGSL or a failed download looked like a working shader. They now fail with the `ShaderError`, as in Rust, Python, Swift, and Kotlin. This is a breaking change. A live editor that should keep drawing keeps the last `Shader` that built, and `Shader.default()` still returns the fallback on request. A path or a registry slug passed to `new Shader` throws on the web as well, since the constructor can't fetch. Use `await Shader.fetch(...)` for those.

Python `Shader.set` names the forms it accepts when a value has the wrong type. Its `TypeError` lists a `Texture`, a bool, `bytes`, `bytearray`, `memoryview`, a sequence of numbers, or a number, where the message used to list the Rust `UniformData` variants. A `TextureMeta` read back with `Shader.get` no longer binds, so pass the `Texture` itself. A value of an accepted form that does not fit the uniform raises `FragmentColorError` under the shared rules.

The Python texture methods check their arguments by type. `Renderer.create_texture`, `Renderer.read_texture`, `Renderer.unregister_texture`, `Texture.write`, `Texture.write_region`, and `Mipmap.build` raise `TypeError` listing the accepted forms when an argument has the wrong type, where they raised `FragmentColorError` before. `Texture.set_sampler_options` names the bad key in its `TypeError`. `Texture.write_region` also accepts a `ScreenRegion`, and any sequence of 2, 4, or 6 ints. Dict regions and dict sampler options still read only the keys they use.

Python raises `FragmentColorError` for renderer and texture errors, as it does for shader and pass errors. A `RendererError`, `InitializationError`, or `TextureError` reached Python as a plain `Exception`, so `except FragmentColorError` missed a failed `Texture.write`, a second `Renderer.unregister_texture`, or a texture target that could not be created, and `Renderer.read_storage` raised `RuntimeError`. Code that catches `Exception` still works, since `FragmentColorError` is a subclass of it.

Python raises `FragmentColorError` subclasses for scene, light, and texture failures. A failed `Scene.load` raises `SceneLoadError` and a light setter that doesn't fit the light raises `LightError`. They raised `RuntimeError` and `ValueError`, which `except FragmentColorError` missed. `SceneLoadError` is also a `RuntimeError` and `LightError` a `ValueError`, so existing `except` clauses still catch them. Texture failures raise `InvalidInputError`, `UnsupportedMipmapFormatError`, or `MalformedImageError`, each a `FragmentColorError`.

Python `Shader(...)` and `Shader.fetch(...)` raise `TypeError` for an argument that is neither a `str` nor a list of `str`, as the texture methods do. They raised `FragmentColorError`.

The Python `Mesh`, `Vertex`, and `Instance` methods check their arguments by type, so the stub lists what each one takes. A position for `Vertex(...)` or `Vertex.pbr(...)` is a `Vertex` to copy, a number, or 2 to 4 numbers. A value for `set` is a `VertexValue`, a number, or 2 to 4 numbers. An instance is an `Instance`, a `Vertex`, or a dict of named values. `Mesh.set_indices` also takes a NumPy array. The stub gains the bare-number vertex and the dict instance, which the methods already took. A position passed as an instance now raises `TypeError`, where it added an instance with no attributes. A one-item list is the number it holds, read as a float, as in the other bindings, where it was the one form that read a plain int as an integer. `VertexValue.u32` and `VertexValue.i32` build integer attributes instead.

Python `TextureTarget.get_image()` returns an array of shape `(height, width, 4)`, indexed `image[y, x]`. It returned `(width, height, 4)` indexed `[x, y]`, the transpose of what `Renderer.create_texture`, `Mipmap.build`, Pillow, and Matplotlib read, so `create_texture(target.get_image())` mirrored the image across its diagonal and swapped the width and height of a target that is not square. This breaks code that indexes the result as `[x, y]`. Swap the two indices, or call `image.transpose(1, 0, 2)` for the old layout. The array's memory now holds the same bytes that `get_image` returns in Rust, JavaScript, Swift, and Kotlin.

JavaScript and Python check sizes, regions, and byte lists with one rule. A size takes 2 or 3 numbers and a texture region 2, 4, or 6, each finite and from 0 up, with a fraction rounded down. A list of the wrong length, a negative or NaN number, a value that isn't a number, or a size dict without `width` and `height` raises the binding's error. JavaScript clamped negative numbers to 0, read a non-number as 0, and clamped bytes outside 0 to 255, while Python turned a list of the wrong length into a 0 by 0 size.

A `ScreenRegion` built from a size alone covers exactly that size. `ScreenRegion::from([w, h])`, `from((w, h))`, `from_tuple`, the JavaScript `[w, h]` form, and Python `ScreenRegion.from_tuple` added one pixel to each side, so `texture.write_region(bytes, ScreenRegion.from_tuple((64, 32)))` targeted 65 by 33 pixels and failed its bounds check. `from_tuples` keeps its two inclusive corners. No renderer code reads a pass viewport yet, so nothing draws differently.

Swift and Kotlin report a GPU that fails to start as `FragmentColorError.Init` from every renderer method. `createTexture`, `createStorageTexture`, `load`, `readStorage`, `readTexture`, and `render` reported it as `Render`. The iOS `createTarget` throws `Init` for a null `CAMetalLayer` pointer, as Android does for a null window, where it read through the pointer.

Kotlin argument errors throw `FragmentColorException`, like every other Kotlin error. This is a behavior change. The extensions checked arguments with `require`, which throws `IllegalArgumentException`, so `catch (e: FragmentColorException)` missed a bad size or a vertex value of the wrong length. Lists of `Int` bytes throw for values outside 0 to 255, which `toByte()` wrapped.

Swift and Kotlin take unsigned types where Rust takes a `u32` or `u64`. Indices, counts, sizes, regions, dispatch sizes, strides, row counts, mip levels, and the unsigned `VertexValue` factories take `UInt32` or `UInt64` in Swift and `UInt` or `ULong` in Kotlin, as the generated methods do. This is a breaking change for code that passes an `Int` variable. The `Int` overloads converted at run time, where Swift trapped on a negative number and Kotlin's `toUInt()` wrapped it to a huge value, and they are gone: Swift's `[Int]` sizes, regions, and indices and its `Int` stride and row count, and Kotlin's `Scene.addTo(Int, ...)`, `Scene.getPass(Int)`, and `Pass.setComputeDispatch(Int, Int, Int)`. Convert a variable with `UInt32(n)` in Swift or `n.toUInt()` in Kotlin, and check its sign first where it can be negative. A Swift literal fits as written, so `scene.getPass(1)` and `renderer.createTextureTarget([64, 64])` compile as before, and Kotlin writes unsigned literals, such as `scene.getPass(1u)` and `listOf(64u, 64u)`. Byte lists keep their `Int` forms.

Kotlin `Shader.setRegistry(baseUrl)` returns nothing, as in Rust, JavaScript, Python, and Swift. This is a behavior change. It returned a default `Shader` built only to be thrown away. Calls that ignore the result compile as before, but the JVM signature changes, so a library compiled against 0.12.5 needs a rebuild. uniffi has no static methods, so both platforms reach the setter through a new generated `Shader.useRegistry`. Swift's labeled `Shader.setRegistry(baseUrl:)` is deprecated in favor of `Shader.setRegistry(_:)`.

The Python stub leaves out binding plumbing. `passes`, `renderable_type`, and the `WindowTarget` protocol methods are gone from `fragmentcolor.pyi`, since dispatch no longer calls them. They still exist until 0.13.

Rust-only internals leave the docs. The device setup functions, the texture and buffer pool types, the default and Shadertoy shader constants, `headless_window`, the glam and wgpu conversions of `UniformData`, and `TextureData::DynamicImage` are hidden from rustdoc and turn crate-private in 0.13.

Python `Texture.get_image` returns a uint8 NumPy array shaped `(height, width, bytes per pixel)`, with a leading layer axis for array textures, the same layout `TextureTarget.get_image` returns. It returned flat `bytes` before. `bytes(image)` still gives the flat bytes, and a float format reads as floats through `image.view(numpy.float32)`. Every other language keeps returning flat bytes in the same row order.

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
- `Light::directional`, `Light::point`, and `Light::spot` follow glTF's `KHR_lights_punctual` model. Each has getters and setters for color, intensity, position, direction, range, and cone angles, and changes reach every shader that uses the light. The PBR shader takes up to 8 lights; more log a warning and are dropped.

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
