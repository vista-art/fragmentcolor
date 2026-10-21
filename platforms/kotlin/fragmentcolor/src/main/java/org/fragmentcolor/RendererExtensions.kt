package org.fragmentcolor

import android.view.Surface

// Idiomatic Kotlin wrappers on top of the uniffi-generated API. Matches
// the method names used by the JavaScript and Python bindings so calling
// code looks identical across every supported platform.

// The raw JNI entry points live in `libfragmentcolor.so` and are exposed
// via a private helper class so the shared library is loaded exactly once.
private object SurfaceBridge {
    init {
        System.loadLibrary("fragmentcolor")
    }

    @JvmStatic
    external fun native_window_from_surface(surface: Surface): Long

    @JvmStatic
    external fun release_native_window(window: Long)
}

/**
 * Build a [MobileWindowTarget] from an [android.view.Surface] on this renderer.
 *
 * JNI reads the surface's `ANativeWindow*`, since uniffi cannot marshal the
 * `JNIEnv*` that step needs. The uniffi `createTarget` then builds the target
 * on this renderer's device. Pairs with the iOS `Renderer.createTarget(layer:)`
 * extension so the same call site compiles on both platforms.
 */
fun Renderer.createTarget(surface: Surface): MobileWindowTarget {
    val window = SurfaceBridge.native_window_from_surface(surface)
    require(window != 0L) { "Failed to acquire an ANativeWindow from Surface" }
    try {
        return createTarget(window.toULong())
    } finally {
        SurfaceBridge.release_native_window(window)
    }
}

/**
 * Create a texture from raw RGBA pixels and their `[width, height]`.
 * The options are Rust's defaults for that size, so the bytes are never decoded.
 */
suspend fun Renderer.createTexture(bytes: ByteArray, size: List<Int>): Texture {
    require(size.size >= 2) { "createTexture: size must be [width, height]" }
    val options = TextureOptions(
        size = Size(width = size[0].toUInt(), height = size[1].toUInt(), depth = null),
        format = TextureFormat.RGBA,
        sampler = SamplerOptions(repeatX = false, repeatY = false, smooth = true, compare = null),
        mipmaps = true,
        usage = null,
    )
    return createTexture(TextureInputMobile.Bytes(bytes), options)
}

/** Create a texture from raw RGBA pixels, one [Int] per byte, and their `[width, height]`. */
suspend fun Renderer.createTexture(bytes: List<Int>, size: List<Int>): Texture =
    createTexture(ByteArray(bytes.size) { bytes[it].toByte() }, size)

/** Create a texture from an image file path or an `http(s)` URL. */
suspend fun Renderer.createTexture(path: String): Texture {
    val input = if (path.startsWith("http://") || path.startsWith("https://")) {
        TextureInputMobile.Url(path)
    } else {
        TextureInputMobile.Path(path)
    }
    return createTexture(input, null)
}

/** Create a texture from a [Mipmap] chain, which uploads without decoding. */
suspend fun Renderer.createTexture(chain: Mipmap): Texture =
    createTexture(TextureInputMobile.Prepared(chain), null)

/** Create a texture from any [TextureInputMobile] with the default options. */
suspend fun Renderer.createTexture(input: TextureInputMobile): Texture = createTexture(input, null)

/**
 * Single overloaded `render(...)` family that matches the spelling used by
 * the JavaScript and Python bindings. The uniffi layer exports one concrete
 * `render(renderable, target)` method that takes `RenderableHandle` +
 * `TargetHandle` enums — these extensions wrap the native types into the
 * matching variants invisibly so callers just write
 * `renderer.render(shader, target)` (or `pass`, `mesh`, `passList`).
 */
fun Renderer.render(shader: Shader, target: MobileWindowTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Window(target))
}

fun Renderer.render(shader: Shader, target: MobileTextureTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Texture(target))
}

fun Renderer.render(pass: Pass, target: MobileWindowTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Window(target))
}

fun Renderer.render(pass: Pass, target: MobileTextureTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Texture(target))
}

fun Renderer.render(mesh: Mesh, target: MobileWindowTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Window(target))
}

fun Renderer.render(mesh: Mesh, target: MobileTextureTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Texture(target))
}

fun Renderer.render(passes: List<Pass>, target: MobileWindowTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Window(target))
}

fun Renderer.render(passes: List<Pass>, target: MobileTextureTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Texture(target))
}

fun Renderer.render(scene: Scene, target: MobileWindowTarget) {
    render(RenderableHandle.Scene(scene), TargetHandle.Window(target))
}

fun Renderer.render(scene: Scene, target: MobileTextureTarget) {
    render(RenderableHandle.Scene(scene), TargetHandle.Texture(target))
}

/**
 * Eager-prewarm overloads matching the `load(...)` family from the JavaScript
 * and Python bindings. The uniffi layer exports one concrete
 * `load(renderable)` taking a [RenderableHandle] enum; these wrap the concrete
 * value into the matching variant so callers write `renderer.load(scene)` or
 * `renderer.load(pass)` directly.
 */
suspend fun Renderer.load(shader: Shader) {
    load(RenderableHandle.Shader(shader))
}

suspend fun Renderer.load(pass: Pass) {
    load(RenderableHandle.Pass(pass))
}

suspend fun Renderer.load(mesh: Mesh) {
    load(RenderableHandle.Mesh(mesh))
}

suspend fun Renderer.load(passes: List<Pass>) {
    load(RenderableHandle.Passes(passes))
}

suspend fun Renderer.load(scene: Scene) {
    load(RenderableHandle.Scene(scene))
}

/**
 * Unregister a texture by its [TextureId] wrapper. Unwraps to the underlying
 * [ULong] handle that the uniffi API expects.
 */
fun Renderer.unregisterTexture(textureId: TextureId) {
    unregisterTexture(textureId.id)
}

/**
 * Read back a registered texture's mip-0 contents by its [TextureId] wrapper.
 * Unwraps to the underlying [ULong] handle that the uniffi API expects so
 * callers can write `renderer.readTexture(texture.id())` directly — matching
 * the spelling used by JavaScript, Python, and Swift bindings.
 */
suspend fun Renderer.readTexture(textureId: TextureId): ByteArray {
    return readTexture(textureId.id)
}
