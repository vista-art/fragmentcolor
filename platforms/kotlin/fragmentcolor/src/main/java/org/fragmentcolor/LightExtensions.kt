package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Light API,
// matching Light+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well.

private fun floats(values: FloatArray): List<Float> = values.toList()

private fun floats(values: List<Double>): List<Float> = values.map { it.toFloat() }

private fun floats(values: DoubleArray): List<Float> = values.map { it.toFloat() }

/** A directional light shining along [direction] in [color]. */
fun Light.Companion.directional(direction: FloatArray, color: FloatArray): Light =
    directional(floats(direction), floats(color))

@JvmName("directionalDoubles")
fun Light.Companion.directional(direction: List<Double>, color: List<Double>): Light =
    directional(floats(direction), floats(color))

fun Light.Companion.directional(direction: DoubleArray, color: DoubleArray): Light =
    directional(floats(direction), floats(color))

/** A point light at [position] in [color]. */
fun Light.Companion.point(position: FloatArray, color: FloatArray): Light = point(floats(position), floats(color))

@JvmName("pointDoubles")
fun Light.Companion.point(position: List<Double>, color: List<Double>): Light = point(floats(position), floats(color))

fun Light.Companion.point(position: DoubleArray, color: DoubleArray): Light = point(floats(position), floats(color))

/** A spot light at [position] shining along [direction] in [color]. */
fun Light.Companion.spot(position: FloatArray, direction: FloatArray, color: FloatArray): Light =
    spot(floats(position), floats(direction), floats(color))

@JvmName("spotDoubles")
fun Light.Companion.spot(position: List<Double>, direction: List<Double>, color: List<Double>): Light =
    spot(floats(position), floats(direction), floats(color))

fun Light.Companion.spot(position: DoubleArray, direction: DoubleArray, color: DoubleArray): Light =
    spot(floats(position), floats(direction), floats(color))

fun Light.setColor(color: FloatArray): Light = setColor(floats(color))

@JvmName("setColorDoubles")
fun Light.setColor(color: List<Double>): Light = setColor(floats(color))

fun Light.setColor(color: DoubleArray): Light = setColor(floats(color))

fun Light.setIntensity(value: Double): Light = setIntensity(value.toFloat())

fun Light.setPosition(position: FloatArray): Light = setPosition(floats(position))

@JvmName("setPositionDoubles")
fun Light.setPosition(position: List<Double>): Light = setPosition(floats(position))

fun Light.setPosition(position: DoubleArray): Light = setPosition(floats(position))

fun Light.setDirection(direction: FloatArray): Light = setDirection(floats(direction))

@JvmName("setDirectionDoubles")
fun Light.setDirection(direction: List<Double>): Light = setDirection(floats(direction))

fun Light.setDirection(direction: DoubleArray): Light = setDirection(floats(direction))

fun Light.setRange(value: Double): Light = setRange(value.toFloat())

fun Light.setConeAngles(innerRadians: Double, outerRadians: Double): Light =
    setConeAngles(innerRadians.toFloat(), outerRadians.toFloat())
