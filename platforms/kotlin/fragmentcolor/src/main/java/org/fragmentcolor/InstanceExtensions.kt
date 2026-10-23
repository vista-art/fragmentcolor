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

// ── VertexValue ───────────────────────────────────────────────────────────────

// Lower-case factories, as JavaScript, Python and Swift spell them.

/** [values] when it holds [size] numbers. Throws [FragmentColorException] otherwise. */
private fun <T> sized(values: List<T>, size: Int, method: String): List<T> {
    if (values.size != size) {
        throw FragmentColorException.Render("$method: expected $size numbers, got ${values.size}")
    }
    return values
}

/** An `f32` value. */
fun VertexValue.Companion.f32(value: Float): VertexValue = VertexValue.F32(value)

/** An `f32` value from a [Double]. */
fun VertexValue.Companion.f32(value: Double): VertexValue = VertexValue.F32(value.toFloat())

/** An `f32` value from an [Int]. */
fun VertexValue.Companion.f32(value: Int): VertexValue = VertexValue.F32(value.toFloat())

/** A `vec2<f32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.f32x2(values: List<Float>): VertexValue = VertexValue.F32x2(sized(values, 2, "VertexValue.f32x2"))

/** A `vec2<f32>` value from [Double]s. */
@JvmName("f32x2Doubles")
fun VertexValue.Companion.f32x2(values: List<Double>): VertexValue = f32x2(values.map { it.toFloat() })

/** A `vec2<f32>` value from a [FloatArray]. */
fun VertexValue.Companion.f32x2(values: FloatArray): VertexValue = f32x2(values.toList())

/** A `vec2<f32>` value from a [DoubleArray]. */
fun VertexValue.Companion.f32x2(values: DoubleArray): VertexValue = f32x2(values.map { it.toFloat() })

/** A `vec3<f32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.f32x3(values: List<Float>): VertexValue = VertexValue.F32x3(sized(values, 3, "VertexValue.f32x3"))

/** A `vec3<f32>` value from [Double]s. */
@JvmName("f32x3Doubles")
fun VertexValue.Companion.f32x3(values: List<Double>): VertexValue = f32x3(values.map { it.toFloat() })

/** A `vec3<f32>` value from a [FloatArray]. */
fun VertexValue.Companion.f32x3(values: FloatArray): VertexValue = f32x3(values.toList())

/** A `vec3<f32>` value from a [DoubleArray]. */
fun VertexValue.Companion.f32x3(values: DoubleArray): VertexValue = f32x3(values.map { it.toFloat() })

/** A `vec4<f32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.f32x4(values: List<Float>): VertexValue = VertexValue.F32x4(sized(values, 4, "VertexValue.f32x4"))

/** A `vec4<f32>` value from [Double]s. */
@JvmName("f32x4Doubles")
fun VertexValue.Companion.f32x4(values: List<Double>): VertexValue = f32x4(values.map { it.toFloat() })

/** A `vec4<f32>` value from a [FloatArray]. */
fun VertexValue.Companion.f32x4(values: FloatArray): VertexValue = f32x4(values.toList())

/** A `vec4<f32>` value from a [DoubleArray]. */
fun VertexValue.Companion.f32x4(values: DoubleArray): VertexValue = f32x4(values.map { it.toFloat() })

/** A `u32` value. */
fun VertexValue.Companion.u32(value: UInt): VertexValue = VertexValue.U32(value)

/** A `vec2<u32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.u32x2(values: List<UInt>): VertexValue = VertexValue.U32x2(sized(values, 2, "VertexValue.u32x2"))

/** A `vec3<u32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.u32x3(values: List<UInt>): VertexValue = VertexValue.U32x3(sized(values, 3, "VertexValue.u32x3"))

/** A `vec4<u32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.u32x4(values: List<UInt>): VertexValue = VertexValue.U32x4(sized(values, 4, "VertexValue.u32x4"))

/** An `i32` value. */
fun VertexValue.Companion.i32(value: Int): VertexValue = VertexValue.I32(value)

/** A `vec2<i32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.i32x2(values: List<Int>): VertexValue = VertexValue.I32x2(sized(values, 2, "VertexValue.i32x2"))

/** A `vec3<i32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.i32x3(values: List<Int>): VertexValue = VertexValue.I32x3(sized(values, 3, "VertexValue.i32x3"))

/** A `vec4<i32>` value. Throws [FragmentColorException] for another length. */
fun VertexValue.Companion.i32x4(values: List<Int>): VertexValue = VertexValue.I32x4(sized(values, 4, "VertexValue.i32x4"))
