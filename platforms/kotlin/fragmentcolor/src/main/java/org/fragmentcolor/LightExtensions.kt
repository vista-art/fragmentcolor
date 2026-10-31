package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Light API,
// matching Light+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well.

private fun floats(values: FloatArray): List<Float> = values.toList()

/** A directional light shining along [direction] in [color]. */
fun Light.Companion.directional(direction: FloatArray, color: FloatArray): Light =
    directional(floats(direction), floats(color))

@JvmName("directionalDoubles")
fun Light.Companion.directional(direction: List<Double>, color: List<Double>): Light =
    directional(readFloats(direction, "Light.directional: direction"), readFloats(color, "Light.directional: color"))

fun Light.Companion.directional(direction: DoubleArray, color: DoubleArray): Light =
    directional(readFloats(direction, "Light.directional: direction"), readFloats(color, "Light.directional: color"))

/** A point light at [position] in [color]. */
fun Light.Companion.point(position: FloatArray, color: FloatArray): Light = point(floats(position), floats(color))

@JvmName("pointDoubles")
fun Light.Companion.point(position: List<Double>, color: List<Double>): Light =
    point(readFloats(position, "Light.point: position"), readFloats(color, "Light.point: color"))

fun Light.Companion.point(position: DoubleArray, color: DoubleArray): Light =
    point(readFloats(position, "Light.point: position"), readFloats(color, "Light.point: color"))

/** A spot light at [position] shining along [direction] in [color]. */
fun Light.Companion.spot(position: FloatArray, direction: FloatArray, color: FloatArray): Light =
    spot(floats(position), floats(direction), floats(color))

@JvmName("spotDoubles")
fun Light.Companion.spot(position: List<Double>, direction: List<Double>, color: List<Double>): Light =
    spot(
        readFloats(position, "Light.spot: position"),
        readFloats(direction, "Light.spot: direction"),
        readFloats(color, "Light.spot: color"),
    )

fun Light.Companion.spot(position: DoubleArray, direction: DoubleArray, color: DoubleArray): Light =
    spot(
        readFloats(position, "Light.spot: position"),
        readFloats(direction, "Light.spot: direction"),
        readFloats(color, "Light.spot: color"),
    )

fun Light.setColor(color: FloatArray): Light = setColor(floats(color))

@JvmName("setColorDoubles")
fun Light.setColor(color: List<Double>): Light = setColor(readFloats(color, "Light.setColor"))

fun Light.setColor(color: DoubleArray): Light = setColor(readFloats(color, "Light.setColor"))

fun Light.setIntensity(value: Double): Light = setIntensity(value.toFloat())

fun Light.setPosition(position: FloatArray): Light = setPosition(floats(position))

@JvmName("setPositionDoubles")
fun Light.setPosition(position: List<Double>): Light = setPosition(readFloats(position, "Light.setPosition"))

fun Light.setPosition(position: DoubleArray): Light = setPosition(readFloats(position, "Light.setPosition"))

fun Light.setDirection(direction: FloatArray): Light = setDirection(floats(direction))

@JvmName("setDirectionDoubles")
fun Light.setDirection(direction: List<Double>): Light = setDirection(readFloats(direction, "Light.setDirection"))

fun Light.setDirection(direction: DoubleArray): Light = setDirection(readFloats(direction, "Light.setDirection"))

fun Light.setRange(value: Double): Light = setRange(value.toFloat())

fun Light.setConeAngles(innerRadians: Double, outerRadians: Double): Light =
    setConeAngles(innerRadians.toFloat(), outerRadians.toFloat())
