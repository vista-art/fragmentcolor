package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Model API,
// matching Model+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well,
// and a 4x4 matrix as its four columns.

/** Replace the model's transform with a column-major 4x4 matrix of 16 numbers. */
fun Model.setTransform(matrix: FloatArray) {
    setTransform(matrix.toList())
}

@JvmName("setTransformDoubles")
fun Model.setTransform(matrix: List<Double>) {
    setTransform(matrix.map { it.toFloat() })
}

fun Model.setTransform(matrix: DoubleArray) {
    setTransform(matrix.map { it.toFloat() })
}

/**
 * Replace the model's transform with a 4x4 matrix given as its four columns, as Rust's `[[f32; 4]; 4]` holds it.
 * Throws [FragmentColorException] unless it holds 4 columns of 4 numbers, naming the column that doesn't.
 */
@JvmName("setTransformColumns")
fun Model.setTransform(columns: List<List<Float>>) {
    setTransform(flattenColumns(columns, "Model.setTransform"))
}

@JvmName("setTransformDoubleColumns")
fun Model.setTransform(columns: List<List<Double>>) {
    val floats = columns.mapIndexed { index, column -> readFloats(column, "Model.setTransform: column $index") }
    setTransform(flattenColumns(floats, "Model.setTransform"))
}

/** A 4x4 matrix from its four columns, flat, column after column, or the error that names the column. */
private fun flattenColumns(columns: List<List<Float>>, label: String): List<Float> {
    if (columns.size != 4) {
        throw FragmentColorException.Render("$label takes 4 columns of 4 numbers, got ${columns.size} columns")
    }
    columns.forEachIndexed { index, column ->
        if (column.size != 4) {
            throw FragmentColorException.Render("$label: column $index takes 4 numbers, got ${column.size}")
        }
    }
    return columns.flatten()
}

fun Model.translate(offset: FloatArray) {
    translate(offset.toList())
}

@JvmName("translateDoubles")
fun Model.translate(offset: List<Double>) {
    translate(readFloats(offset, "Model.translate"))
}

fun Model.translate(offset: DoubleArray) {
    translate(readFloats(offset, "Model.translate"))
}

fun Model.rotate(axis: FloatArray, radians: Float) {
    rotate(axis.toList(), radians)
}

@JvmName("rotateDoubles")
fun Model.rotate(axis: List<Double>, radians: Double) {
    rotate(readFloats(axis, "Model.rotate: axis"), radians.toFloat())
}

fun Model.rotate(axis: DoubleArray, radians: Double) {
    rotate(readFloats(axis, "Model.rotate: axis"), radians.toFloat())
}

fun Model.scale(factor: FloatArray) {
    scale(factor.toList())
}

@JvmName("scaleDoubles")
fun Model.scale(factor: List<Double>) {
    scale(readFloats(factor, "Model.scale"))
}

fun Model.scale(factor: DoubleArray) {
    scale(readFloats(factor, "Model.scale"))
}
