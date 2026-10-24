package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Instance and Vertex APIs.
// The generated `set(key, List<Double>)` takes 1 to 4 numbers and throws
// [FragmentColorException] for any other length. These overloads take the other
// number types and pass them on unchanged. The shader that reads an attribute
// decides its type when the mesh attaches, so `7` fills an f32, i32 or u32 input.

// ── Instance ──────────────────────────────────────────────────────────────────

/** Set an instance attribute to a [Float]. */
fun Instance.set(key: String, value: Float): Instance = set(key, listOf(value.toDouble()))

/** Set an instance attribute to a [Double]. */
fun Instance.set(key: String, value: Double): Instance = set(key, listOf(value))

/** Set an instance attribute to an [Int]. */
fun Instance.set(key: String, value: Int): Instance = set(key, listOf(value.toDouble()))

/** Set an instance attribute to a [Long]. */
fun Instance.set(key: String, value: Long): Instance = set(key, listOf(value.toDouble()))

/** Set an instance attribute to a [UInt]. */
fun Instance.set(key: String, value: UInt): Instance = set(key, listOf(value.toDouble()))

/** Set an instance attribute to the numbers of a [FloatArray]. */
fun Instance.set(key: String, values: FloatArray): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to the numbers of a [DoubleArray]. */
fun Instance.set(key: String, values: DoubleArray): Instance = set(key, values.toList())

/** Set an instance attribute to the numbers of an [IntArray]. */
fun Instance.set(key: String, values: IntArray): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to the numbers of a [LongArray]. */
fun Instance.set(key: String, values: LongArray): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to the numbers of a [UIntArray]. */
@OptIn(ExperimentalUnsignedTypes::class)
fun Instance.set(key: String, values: UIntArray): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to a list of [Float]s. */
@JvmName("setFloats")
fun Instance.set(key: String, values: List<Float>): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to a list of [Int]s. */
@JvmName("setInts")
fun Instance.set(key: String, values: List<Int>): Instance = set(key, values.map { it.toDouble() })

/** Set an instance attribute to a list of [UInt]s. */
@JvmName("setUInts")
fun Instance.set(key: String, values: List<UInt>): Instance = set(key, values.map { it.toDouble() })

// ── Vertex ────────────────────────────────────────────────────────────────────

/** Set a vertex attribute to a [Float]. */
fun Vertex.set(key: String, value: Float): Vertex = set(key, listOf(value.toDouble()))

/** Set a vertex attribute to a [Double]. */
fun Vertex.set(key: String, value: Double): Vertex = set(key, listOf(value))

/** Set a vertex attribute to an [Int]. */
fun Vertex.set(key: String, value: Int): Vertex = set(key, listOf(value.toDouble()))

/** Set a vertex attribute to a [Long]. */
fun Vertex.set(key: String, value: Long): Vertex = set(key, listOf(value.toDouble()))

/** Set a vertex attribute to a [UInt]. */
fun Vertex.set(key: String, value: UInt): Vertex = set(key, listOf(value.toDouble()))

/** Set a vertex attribute to the numbers of a [FloatArray]. */
fun Vertex.set(key: String, values: FloatArray): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to the numbers of a [DoubleArray]. */
fun Vertex.set(key: String, values: DoubleArray): Vertex = set(key, values.toList())

/** Set a vertex attribute to the numbers of an [IntArray]. */
fun Vertex.set(key: String, values: IntArray): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to the numbers of a [LongArray]. */
fun Vertex.set(key: String, values: LongArray): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to the numbers of a [UIntArray]. */
@OptIn(ExperimentalUnsignedTypes::class)
fun Vertex.set(key: String, values: UIntArray): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to a list of [Float]s. */
@JvmName("setFloats")
fun Vertex.set(key: String, values: List<Float>): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to a list of [Int]s. */
@JvmName("setInts")
fun Vertex.set(key: String, values: List<Int>): Vertex = set(key, values.map { it.toDouble() })

/** Set a vertex attribute to a list of [UInt]s. */
@JvmName("setUInts")
fun Vertex.set(key: String, values: List<UInt>): Vertex = set(key, values.map { it.toDouble() })
