package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Texture API,
// matching the call shapes of Texture+Extensions.swift.

/**
 * Upload pixels into part of the texture, given as `[width, height]`,
 * `[x, y, width, height]` or `[x, y, z, width, height, depth]`.
 */
fun Texture.writeRegion(bytes: ByteArray, rect: List<Int>) {
    writeRegion(bytes, TextureRegionMobile.from(rect))
}

/** Upload pixels, one [Int] per byte, into part of the texture given as a list of numbers. */
fun Texture.writeRegion(bytes: List<Int>, rect: List<Int>) {
    writeRegion(ByteArray(bytes.size) { bytes[it].toByte() }, rect)
}

/** Upload pixels, one [Int] per byte, into a [TextureRegionMobile]. */
fun Texture.writeRegion(bytes: List<Int>, region: TextureRegionMobile) {
    writeRegion(ByteArray(bytes.size) { bytes[it].toByte() }, region)
}
