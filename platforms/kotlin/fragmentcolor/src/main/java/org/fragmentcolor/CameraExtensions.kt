package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Camera API,
// matching Camera+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well,
// so a literal such as `1.047` or `floatArrayOf(0f, 0f, 5f)` goes straight in.

/** A perspective camera, with every value a [Double]. */
fun Camera.Companion.perspective(fovyRadians: Double, aspect: Double, near: Double, far: Double): Camera =
    perspective(fovyRadians.toFloat(), aspect.toFloat(), near.toFloat(), far.toFloat())

/** An orthographic camera, with every value a [Double]. */
fun Camera.Companion.orthographic(
    left: Double,
    right: Double,
    bottom: Double,
    top: Double,
    near: Double,
    far: Double,
): Camera = orthographic(left.toFloat(), right.toFloat(), bottom.toFloat(), top.toFloat(), near.toFloat(), far.toFloat())

/** Aim the camera from [position] at [target], with [up] as the up direction. */
fun Camera.lookAt(position: FloatArray, target: FloatArray, up: FloatArray): Camera =
    lookAt(position.toList(), target.toList(), up.toList())

/** Aim the camera from [position] at [target], with [up] as the up direction. */
@JvmName("lookAtDoubles")
fun Camera.lookAt(position: List<Double>, target: List<Double>, up: List<Double>): Camera =
    lookAt(position.map { it.toFloat() }, target.map { it.toFloat() }, up.map { it.toFloat() })

/** Aim the camera from [position] at [target], with [up] as the up direction. */
fun Camera.lookAt(position: DoubleArray, target: DoubleArray, up: DoubleArray): Camera =
    lookAt(position.toList(), target.toList(), up.toList())

/** Set the width-to-height ratio. */
fun Camera.setAspect(aspect: Double): Camera = setAspect(aspect.toFloat())
