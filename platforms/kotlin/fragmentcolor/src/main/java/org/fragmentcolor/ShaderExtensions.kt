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

// Each overload only says what kind of value it carries. Rust shapes it to
// the uniform's declared WGSL type, so `3` fits an f32, i32 or u32 uniform
// and four numbers fit a vec4 or a mat2x2. The generated member
// `set(key, UniformValue)` wins inside each body, so there is no recursion.

fun Shader.set(key: String, value: Float) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: Double) = set(key, UniformValue.Number(value))

fun Shader.set(key: String, value: Int) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: UInt) = set(key, UniformValue.Number(value.toDouble()))

fun Shader.set(key: String, value: Boolean) = set(key, UniformValue.Bool(value))

fun Shader.set(key: String, value: FloatArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

fun Shader.set(key: String, value: DoubleArray) = set(key, UniformValue.Numbers(value.toList()))

fun Shader.set(key: String, value: IntArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

fun Shader.set(key: String, value: UIntArray) =
    set(key, UniformValue.Numbers(value.map { it.toDouble() }))

/** Raw bytes for a storage-buffer or push-constant root. */
fun Shader.set(key: String, bytes: ByteArray) = set(key, UniformValue.Bytes(bytes))

/** Bind a [Texture] created by a [Renderer] to a texture uniform. */
fun Shader.set(key: String, texture: Texture) = set(key, UniformValue.Texture(texture.id()))

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
