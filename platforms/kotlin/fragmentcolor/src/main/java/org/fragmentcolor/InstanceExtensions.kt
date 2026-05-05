package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Instance and Vertex APIs.
// Provides natural overloads so callers can pass List<Float> / FloatArray instead of
// constructing VertexValue variants by hand — matches the Swift and JS bindings.

// ── Instance ──────────────────────────────────────────────────────────────────

/** Set an instance attribute from a [FloatArray]. Dispatches to F32 / F32x2..4 by length. */
fun Instance.set(key: String, value: FloatArray): Instance {
    val list = value.toList()
    val v: VertexValue = when (value.size) {
        1 -> VertexValue.F32(list[0])
        2 -> VertexValue.F32x2(list)
        3 -> VertexValue.F32x3(list)
        4 -> VertexValue.F32x4(list)
        else -> throw IllegalArgumentException("Unsupported float array length: ${value.size}")
    }
    return set(key, v)
}

/** Set an instance attribute from a [List]<[Float]>. Dispatches to F32 / F32x2..4 by length. */
fun Instance.set(key: String, value: List<Float>): Instance = set(key, value.toFloatArray())

// ── Vertex ────────────────────────────────────────────────────────────────────

/** Set a vertex attribute from a [FloatArray]. Dispatches to F32 / F32x2..4 by length. */
fun Vertex.set(key: String, value: FloatArray): Vertex {
    val list = value.toList()
    val v: VertexValue = when (value.size) {
        1 -> VertexValue.F32(list[0])
        2 -> VertexValue.F32x2(list)
        3 -> VertexValue.F32x3(list)
        4 -> VertexValue.F32x4(list)
        else -> throw IllegalArgumentException("Unsupported float array length: ${value.size}")
    }
    return set(key, v)
}

/** Set a vertex attribute from a [List]<[Float]>. Dispatches to F32 / F32x2..4 by length. */
fun Vertex.set(key: String, value: List<Float>): Vertex = set(key, value.toFloatArray())

/** Set a vertex attribute from a scalar [Float]. */
fun Vertex.set(key: String, value: Float): Vertex = set(key, VertexValue.F32(value))

/** Set a vertex attribute from a scalar [Double] (convenience; converts to Float). */
fun Vertex.set(key: String, value: Double): Vertex = set(key, VertexValue.F32(value.toFloat()))

/** Set an instance attribute from a scalar [Float]. */
fun Instance.set(key: String, value: Float): Instance = set(key, VertexValue.F32(value))

/** Set an instance attribute from a scalar [Double] (convenience; converts to Float). */
fun Instance.set(key: String, value: Double): Instance = set(key, VertexValue.F32(value.toFloat()))
