package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Shader API.
// Matches the call shapes used by the JavaScript and Python bindings so
// cross-platform examples read the same on every platform.

/** Point registry slugs at [baseUrl], for every Shader in the process. */
fun Shader.Companion.setRegistry(baseUrl: String) {
    useRegistry(baseUrl).destroy()
}

/** Build a [Shader] from parts: WGSL sources, registry slugs, URLs or file paths. */
operator fun Shader.Companion.invoke(parts: List<String>): Shader = Shader.compose(parts)

// Each overload only says what kind of value it carries. Rust shapes it to
// the uniform's declared WGSL type, so `3` fits an f32, i32 or u32 uniform
// and four numbers fit a vec4 or a mat2x2. The generated member
// `set(key, UniformValue)` wins inside each body, so there is no recursion.

fun Shader.set(key: String, value: Float) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: Double) = set(key, UniformValue.Number(value))

fun Shader.set(key: String, value: Int) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: Long) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: UInt) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: Boolean) = set(key, UniformValue.Bool(value))

fun Shader.set(key: String, value: FloatArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

fun Shader.set(key: String, value: DoubleArray) = set(key, UniformValue.Numbers(value.toList()))

fun Shader.set(key: String, value: IntArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

fun Shader.set(key: String, value: LongArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

@OptIn(ExperimentalUnsignedTypes::class)
fun Shader.set(key: String, value: UIntArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

/** A vector or matrix, such as the matrix `Camera.viewProj()` returns. */
@JvmName("setFloats")
fun Shader.set(key: String, value: List<Float>) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

@JvmName("setDoubles")
fun Shader.set(key: String, value: List<Double>) = set(key, UniformValue.Numbers(value))

@JvmName("setInts")
fun Shader.set(key: String, value: List<Int>) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

@JvmName("setUInts")
fun Shader.set(key: String, value: List<UInt>) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

/** A matrix as its columns, as Rust's `[[f32; 4]; 4]` holds it. */
@JvmName("setFloatColumns")
fun Shader.set(key: String, columns: List<List<Float>>) = set(key, columns.flatten())

/** A matrix as its columns. */
@JvmName("setDoubleColumns")
fun Shader.set(key: String, columns: List<List<Double>>) = set(key, columns.flatten())

/** A matrix as its columns. */
@JvmName("setFloatArrayColumns")
fun Shader.set(key: String, columns: List<FloatArray>) = set(key, columns.flatMap { it.toList() })

/** Raw bytes for a storage-buffer or push-constant root. */
fun Shader.set(key: String, bytes: ByteArray) = set(key, UniformValue.Bytes(bytes))

/** Bind a [Texture] created by a [Renderer] to a texture uniform. */
fun Shader.set(key: String, texture: Texture) = set(key, UniformValue.Texture(texture.id()))

// `Shader.get` returns a [UniformData]. These accessors read it.

/**
 * The number a scalar uniform holds, as a [Float].
 * `null` for anything but a bool, float, int or uint.
 */
val UniformData.float: Float?
    get() = when (this) {
        is UniformData.Float -> v1
        is UniformData.Int -> v1.toFloat()
        is UniformData.UInt -> v1.toFloat()
        is UniformData.Bool -> if (v1) 1f else 0f
        else -> null
    }

/**
 * The numbers a scalar, vector or matrix uniform holds, as [Float]s.
 * Matrices come column by column. `null` for textures, samplers and bytes.
 */
val UniformData.floats: List<Float>?
    get() = when (this) {
        is UniformData.Vec2 -> v1
        is UniformData.Vec3 -> v1
        is UniformData.Vec4 -> v1
        is UniformData.Mat2 -> v1
        is UniformData.Mat3 -> v1
        is UniformData.Mat4 -> v1
        is UniformData.UVec2 -> v1.map { it.toFloat() }
        is UniformData.UVec3 -> v1.map { it.toFloat() }
        is UniformData.UVec4 -> v1.map { it.toFloat() }
        is UniformData.IVec2 -> v1.map { it.toFloat() }
        is UniformData.IVec3 -> v1.map { it.toFloat() }
        is UniformData.IVec4 -> v1.map { it.toFloat() }
        else -> float?.let { listOf(it) }
    }

/** The bytes of a storage-buffer or push-constant root. `null` for anything else. */
val UniformData.bytes: ByteArray?
    get() = (this as? UniformData.Bytes)?.v1

// ---------------------------------------------------------------------------
// Async fetch factory helpers
// ---------------------------------------------------------------------------

@Deprecated("Use Shader.fetch(input)", ReplaceWith("Shader.fetch(input)"))
suspend fun ShaderFetch(input: String): Shader = Shader.fetch(input)

@Deprecated("Use Shader.fetch(parts)", ReplaceWith("Shader.fetch(parts)"))
suspend fun ShaderFetch(parts: List<String>): Shader = Shader.fetch(parts)

// Companion-object extensions that mirror the static-factory spelling used by
// the JavaScript, Python, and Swift bindings: `Shader.fetch(input)` /
// `Shader.fetch(parts)`. The underlying binding is an instance method on
// `Shader`; these wrappers create a throw-away default instance and discard
// it after the fetch.

/**
 * Static-factory async fetch: resolve a single URL, slug, file path, or raw
 * WGSL source and return a compiled [Shader]. Matches `Shader.fetch(...)` on
 * JavaScript, Python, and Swift.
 *
 * Usage:
 * ```kotlin
 * val shader = Shader.fetch("https://example.com/shader.wgsl")
 * ```
 */
suspend fun Shader.Companion.fetch(input: String): Shader =
    Shader.default().fetch(input)

/**
 * Static-factory async fetch (multi-part): resolve each entry asynchronously,
 * merge them, and return a compiled [Shader].
 *
 * Usage:
 * ```kotlin
 * val shader = Shader.fetch(listOf("sdf2d/circle", mySource))
 * ```
 */
suspend fun Shader.Companion.fetch(parts: List<String>): Shader =
    Shader.default().fetchCompose(parts)
