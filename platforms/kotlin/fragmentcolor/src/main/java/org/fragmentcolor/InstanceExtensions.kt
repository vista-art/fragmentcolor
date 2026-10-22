package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Instance and Vertex APIs.
// Provides natural overloads so callers can pass List<Float> / FloatArray instead of
// constructing VertexValue variants by hand — matches the Swift and JS bindings.

/**
 * The `f32` vertex value of 1 to 4 floats.
 * Throws [FragmentColorException] for any other length.
 */
private fun floatValue(value: List<Float>, method: String): VertexValue = when (value.size) {
    1 -> VertexValue.F32(value[0])
    2 -> VertexValue.F32x2(value)
    3 -> VertexValue.F32x3(value)
    4 -> VertexValue.F32x4(value)
    else -> throw FragmentColorException.Render("$method: expected 1 to 4 numbers, got ${value.size}")
}

// ── Instance ──────────────────────────────────────────────────────────────────

/** Set an instance attribute from a [FloatArray]. Dispatches to F32 / F32x2..4 by length. */
fun Instance.set(key: String, value: FloatArray): Instance = set(key, value.toList())

/** Set an instance attribute from a [List]<[Float]>. Dispatches to F32 / F32x2..4 by length. */
fun Instance.set(key: String, value: List<Float>): Instance = set(key, floatValue(value, "Instance.set"))

/** Set an instance attribute from a [DoubleArray]. */
fun Instance.set(key: String, value: DoubleArray): Instance = set(key, value.map { it.toFloat() })

/** Set an instance attribute from a [List]<[Double]>. */
@JvmName("setDoubles")
fun Instance.set(key: String, value: List<Double>): Instance = set(key, value.map { it.toFloat() })

/** Set an instance attribute from a scalar [Float]. */
fun Instance.set(key: String, value: Float): Instance = set(key, VertexValue.F32(value))

/** Set an instance attribute from a scalar [Double] (convenience; converts to Float). */
fun Instance.set(key: String, value: Double): Instance = set(key, VertexValue.F32(value.toFloat()))

// ── Vertex ────────────────────────────────────────────────────────────────────

/** Set a vertex attribute from a [FloatArray]. Dispatches to F32 / F32x2..4 by length. */
fun Vertex.set(key: String, value: FloatArray): Vertex = set(key, value.toList())

/** Set a vertex attribute from a [List]<[Float]>. Dispatches to F32 / F32x2..4 by length. */
fun Vertex.set(key: String, value: List<Float>): Vertex = set(key, floatValue(value, "Vertex.set"))

/** Set a vertex attribute from a [DoubleArray]. */
fun Vertex.set(key: String, value: DoubleArray): Vertex = set(key, value.map { it.toFloat() })

/** Set a vertex attribute from a [List]<[Double]>. */
@JvmName("setDoubles")
fun Vertex.set(key: String, value: List<Double>): Vertex = set(key, value.map { it.toFloat() })

/** Set a vertex attribute from a scalar [Float]. */
fun Vertex.set(key: String, value: Float): Vertex = set(key, VertexValue.F32(value))

/** Set a vertex attribute from a scalar [Double] (convenience; converts to Float). */
fun Vertex.set(key: String, value: Double): Vertex = set(key, VertexValue.F32(value.toFloat()))
