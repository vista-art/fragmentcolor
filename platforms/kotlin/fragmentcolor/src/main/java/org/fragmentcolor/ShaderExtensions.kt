package org.fragmentcolor

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

// Idiomatic Kotlin wrappers on top of the uniffi-generated Shader API.
// Matches the call shapes used by the JavaScript and Python bindings so
// cross-platform examples read the same on every platform.

// Shader.setRegistry(baseUrl) is provided directly by the uniffi-generated
// companion object — calling it returns a Shader that the caller may discard.
// We deliberately do NOT define a Void-returning extension here because such
// an extension would shadow the generated method and cause infinite recursion
// when this file calls Shader.setRegistry(baseUrl) (the call resolves to the
// shadowing extension, not the underlying companion method).

/** Set a Float uniform. */
fun Shader.set(key: String, value: Float) {
    set(key, UniformData.Float(value))
}

/**
 * Set a Float uniform from a Kotlin [Double] literal. Lets example code
 * write `shader.set("k", 0.5)` without the `f` suffix; the value is downcast
 * to f32 before the FFI hop, matching every other binding's behaviour.
 */
fun Shader.set(key: String, value: Double) {
    set(key, UniformData.Float(value.toFloat()))
}

/** Set an Int (i32) uniform. */
fun Shader.set(key: String, value: Int) {
    set(key, UniformData.Int(value))
}

/** Set a UInt (u32) uniform. */
fun Shader.set(key: String, value: UInt) {
    set(key, UniformData.UInt(value))
}

/** Set a Bool uniform. */
fun Shader.set(key: String, value: Boolean) {
    set(key, UniformData.Bool(value))
}

/** Float array — dispatches to Float / Vec2..4 / Mat3 / Mat4 by length. */
fun Shader.set(key: String, value: FloatArray) {
    val list = value.toList()
    val uniform = when (value.size) {
        1 -> UniformData.Float(list[0])
        2 -> UniformData.Vec2(list)
        3 -> UniformData.Vec3(list)
        4 -> UniformData.Vec4(list)
        9 -> UniformData.Mat3(list)
        16 -> UniformData.Mat4(list)
        else -> throw FragmentColorException.Shader(
            "Unsupported float array length: ${value.size} (expected 1/2/3/4/9/16)"
        )
    }
    set(key, uniform)
}

/** Int (i32) array — dispatches to Int / IVec2..4 by length. */
fun Shader.set(key: String, value: IntArray) {
    val list = value.toList()
    val uniform = when (value.size) {
        1 -> UniformData.Int(list[0])
        2 -> UniformData.IVec2(list)
        3 -> UniformData.IVec3(list)
        4 -> UniformData.IVec4(list)
        else -> throw FragmentColorException.Shader(
            "Unsupported int array length: ${value.size} (expected 1/2/3/4)"
        )
    }
    set(key, uniform)
}

/** UInt (u32) array — dispatches to UInt / UVec2..4 by length. */
fun Shader.set(key: String, value: UIntArray) {
    val list = value.toList()
    val uniform = when (value.size) {
        1 -> UniformData.UInt(list[0])
        2 -> UniformData.UVec2(list)
        3 -> UniformData.UVec3(list)
        4 -> UniformData.UVec4(list)
        else -> throw FragmentColorException.Shader(
            "Unsupported uint array length: ${value.size} (expected 1/2/3/4)"
        )
    }
    set(key, uniform)
}

/**
 * Pass a [Texture] handle for a sampler/texture binding. Storage merges
 * the user-supplied id with the shader-parsed metadata at set time, so
 * the placeholder values for `dim` / `arrayed` / `class` / `sampled`
 * here are overwritten by the real reflection data.
 */
fun Shader.set(key: String, texture: Texture) {
    val meta = TextureMeta(
        id = texture.id(),
        dim = TextureDim.D2,
        arrayed = false,
        `class` = TextureClass.Sampled(TextureScalarKind.FLOAT, false),
        sampled = true
    )
    set(key, UniformData.Texture(meta))
}

// ---------------------------------------------------------------------------
// Async fetch factory helpers
// ---------------------------------------------------------------------------

/**
 * Resolve a single URL, slug, file path, or raw WGSL source asynchronously
 * and return a compiled [Shader]. Mirrors `Shader.fetch(...)` on Web and Python.
 *
 * The underlying uniffi binding exposes `fetch` as a `suspend` instance method
 * (uniffi 0.31 does not support async constructors); this companion-style
 * top-level function creates a throw-away default instance to call through.
 *
 * Usage:
 * ```kotlin
 * val shader = ShaderFetch("https://example.com/shader.wgsl")
 * ```
 */
suspend fun ShaderFetch(input: String): Shader =
    Shader.default().fetch(input)

/**
 * Resolve an array of parts (URLs, slugs, paths, raw WGSL) asynchronously,
 * merge them, and return a compiled [Shader].
 *
 * Usage:
 * ```kotlin
 * val shader = ShaderFetch(listOf("sdf2d/circle", mySource))
 * ```
 */
suspend fun ShaderFetch(parts: List<String>): Shader =
    Shader.default().fetchCompose(parts)

// Companion-object extensions that mirror the static-factory spelling used by
// the JavaScript, Python, and Swift bindings: `Shader.fetch(input)` /
// `Shader.fetch(parts)`. uniffi 0.31 cannot generate async constructors, so
// the underlying binding is an instance method on `Shader`; these wrappers
// create a throw-away default instance and discard it after the fetch.

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
