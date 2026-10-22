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
 * Build a [WindowTarget] from an [android.view.Surface] on this renderer.
 *
 * JNI reads the surface's `ANativeWindow*`, since uniffi cannot marshal the
 * `JNIEnv*` that step needs. The uniffi `createTarget` then builds the target
 * on this renderer's device. Pairs with the iOS `Renderer.createTarget(layer:)`
 * extension so the same call site compiles on both platforms.
 */
fun Renderer.createTarget(surface: Surface): WindowTarget {
    val window = SurfaceBridge.native_window_from_surface(surface)
    if (window == 0L) {
        throw FragmentColorException.Init("createTarget: the Surface has no ANativeWindow")
    }
    try {
        return createTarget(window.toULong())
    } finally {
        SurfaceBridge.release_native_window(window)
    }
}

// ── createTextureTarget / createDepthTexture ─────────────────────────────────

/** Create a headless texture target from a `[width, height]` list. */
suspend fun Renderer.createTextureTarget(size: List<Int>): TextureTarget =
    createTextureTarget(sizeRecord(size, "createTextureTarget"))

/** Create a headless texture target from a [Size]. */
suspend fun Renderer.createTextureTarget(size: Size): TextureTarget =
    createTextureTarget(size.width, size.height)

/** Create a depth texture from a `[width, height]` list. */
suspend fun Renderer.createDepthTexture(size: List<Int>): Texture =
    createDepthTexture(sizeRecord(size, "createDepthTexture"))

/** Create a depth texture from a [Size]. */
suspend fun Renderer.createDepthTexture(size: Size): Texture = createDepthTexture(size.width, size.height)

// ── createStorageTexture ─────────────────────────────────────────────────────

/**
 * Create a storage texture of [size] and [format], optionally seeded with [data].
 * [usage] replaces the default `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`.
 */
suspend fun Renderer.createStorageTexture(
    size: Size,
    format: TextureFormat,
    data: ByteArray? = null,
    usage: TextureUsage? = null,
): Texture = createStorageTexture(size, format, data, usage?.bits)

/** Create a storage texture from a `[width, height]` or `[width, height, depth]` list. */
suspend fun Renderer.createStorageTexture(
    size: List<Int>,
    format: TextureFormat,
    data: ByteArray? = null,
    usage: TextureUsage? = null,
): Texture = createStorageTexture(sizeRecord(size, "createStorageTexture"), format, data, usage)

/** Create a storage texture seeded with one [Int] per byte. */
@JvmName("createStorageTextureInts")
suspend fun Renderer.createStorageTexture(
    size: List<Int>,
    format: TextureFormat,
    data: List<Int>,
    usage: TextureUsage? = null,
): Texture = createStorageTexture(size, format, byteArray(data, "createStorageTexture"), usage)

// ── createTexture ────────────────────────────────────────────────────────────

/**
 * Create a texture from encoded image bytes (PNG, JPEG and the other formats FragmentColor decodes).
 * [usage] adds to the `TEXTURE_BINDING | COPY_DST` every texture gets.
 */
suspend fun Renderer.createTexture(bytes: ByteArray, usage: TextureUsage? = null): Texture =
    createTexture(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

/** Create a texture from encoded image bytes, decoded into [format]. */
suspend fun Renderer.createTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Texture =
    createTexture(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

/** Create a texture from raw RGBA pixels and their [Size], so the bytes are never decoded. */
suspend fun Renderer.createTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Texture =
    createTexture(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

/** Create a texture from raw RGBA pixels and their `[width, height]`. */
suspend fun Renderer.createTexture(bytes: ByteArray, size: List<Int>, usage: TextureUsage? = null): Texture =
    createTexture(bytes, sizeRecord(size, "createTexture"), usage)

/** Create a texture from raw RGBA pixels, one [Int] per byte, and their `[width, height]`. */
suspend fun Renderer.createTexture(bytes: List<Int>, size: List<Int>, usage: TextureUsage? = null): Texture =
    createTexture(byteArray(bytes, "createTexture"), size, usage)

/** Create a texture from bytes with every option spelled out. */
suspend fun Renderer.createTexture(bytes: ByteArray, options: TextureOptions): Texture =
    createTexture(TextureInputMobile.Bytes(bytes), options)

/** Create a texture from an image file path or an `http(s)` URL. */
suspend fun Renderer.createTexture(path: String): Texture = createTexture(pathInput(path), null)

/** Create a texture from an image file path or an `http(s)` URL, with every option spelled out. */
suspend fun Renderer.createTexture(path: String, options: TextureOptions): Texture =
    createTexture(pathInput(path), options)

/** Create a texture from a [Mipmap] chain, which uploads without decoding. */
suspend fun Renderer.createTexture(chain: Mipmap): Texture =
    createTexture(TextureInputMobile.Prepared(chain), null)

/** Create a texture from any [TextureInputMobile] with the default options. */
suspend fun Renderer.createTexture(input: TextureInputMobile): Texture = createTexture(input, null)

// ── render / load ────────────────────────────────────────────────────────────

/**
 * Single overloaded `render(...)` family that matches the spelling used by
 * the JavaScript and Python bindings. The uniffi layer exports one concrete
 * `render(renderable, target)` method that takes `RenderableHandle` +
 * `TargetHandle` enums — these extensions wrap the native types into the
 * matching variants invisibly so callers just write
 * `renderer.render(shader, target)` (or `pass`, `mesh`, `passList`).
 */
fun Renderer.render(shader: Shader, target: WindowTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Window(target))
}

fun Renderer.render(shader: Shader, target: TextureTarget) {
    render(RenderableHandle.Shader(shader), TargetHandle.Texture(target))
}

fun Renderer.render(pass: Pass, target: WindowTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Window(target))
}

fun Renderer.render(pass: Pass, target: TextureTarget) {
    render(RenderableHandle.Pass(pass), TargetHandle.Texture(target))
}

fun Renderer.render(mesh: Mesh, target: WindowTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Window(target))
}

fun Renderer.render(mesh: Mesh, target: TextureTarget) {
    render(RenderableHandle.Mesh(mesh), TargetHandle.Texture(target))
}

fun Renderer.render(passes: List<Pass>, target: WindowTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Window(target))
}

fun Renderer.render(passes: List<Pass>, target: TextureTarget) {
    render(RenderableHandle.Passes(passes), TargetHandle.Texture(target))
}

fun Renderer.render(scene: Scene, target: WindowTarget) {
    render(RenderableHandle.Scene(scene), TargetHandle.Window(target))
}

fun Renderer.render(scene: Scene, target: TextureTarget) {
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

/**
 * A [Size] from `[width, height]` or `[width, height, depth]`.
 * Throws [FragmentColorException] for any other length or a negative number.
 */
internal fun sizeRecord(size: List<Int>, method: String): Size {
    if (size.size != 2 && size.size != 3) {
        throw FragmentColorException.Render("$method: a size holds 2 or 3 numbers, got ${size.size}")
    }
    val dims = size.map { unsigned(it, method, "size") }
    return Size(width = dims[0], height = dims[1], depth = dims.getOrNull(2))
}

/** [value] as a [UInt]. Throws [FragmentColorException] for a negative number. */
internal fun unsigned(value: Int, method: String, what: String): UInt {
    if (value < 0) {
        throw FragmentColorException.Render("$method: $value is not a valid $what")
    }
    return value.toUInt()
}

/** Bytes from one [Int] per byte. Throws [FragmentColorException] for a value outside 0..255. */
internal fun byteArray(bytes: List<Int>, method: String): ByteArray =
    ByteArray(bytes.size) { index ->
        val value = bytes[index]
        if (value !in 0..255) {
            throw FragmentColorException.Render("$method: $value is not a byte")
        }
        value.toByte()
    }

/** A URL input for `http(s)`, otherwise a file path. */
internal fun pathInput(path: String): TextureInputMobile =
    if (path.startsWith("http://") || path.startsWith("https://")) {
        TextureInputMobile.Url(path)
    } else {
        TextureInputMobile.Path(path)
    }

/**
 * Rust's default [TextureOptions] with an optional size, format and usage,
 * or `null` when nothing differs, so the renderer applies its own defaults.
 */
internal fun textureOptions(
    size: Size? = null,
    format: TextureFormat? = null,
    usage: TextureUsage? = null,
): TextureOptions? {
    if (size == null && format == null && usage == null) {
        return null
    }
    return TextureOptions(
        size = size,
        format = format ?: TextureFormat.RGBA,
        sampler = SamplerOptions(repeatX = false, repeatY = false, smooth = true, compare = null),
        mipmaps = true,
        usage = usage?.bits,
    )
}
