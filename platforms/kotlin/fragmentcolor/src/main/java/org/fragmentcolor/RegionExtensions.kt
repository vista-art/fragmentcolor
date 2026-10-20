package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated TextureRegionMobile
// record. Rust builds a TextureRegion through from([x, y, w, h]), with_stride
// and with_rows. The uniffi record is plain fields, so these functions
// reproduce the Rust call shape, as Region+Extensions.swift does on Swift.

/**
 * A texture region from `[width, height]`, `[x, y, width, height]`
 * or `[x, y, z, width, height, depth]`, as Rust's `TextureRegion::from` reads them.
 */
fun TextureRegionMobile.Companion.from(rect: List<Int>): TextureRegionMobile {
    val r = rect.map { it.toUInt() }
    val (origin, size) = when (r.size) {
        2 -> listOf(0u, 0u, 0u) to listOf(r[0], r[1], 1u)
        4 -> listOf(r[0], r[1], 0u) to listOf(r[2], r[3], 1u)
        6 -> r.subList(0, 3) to r.subList(3, 6)
        else -> throw IllegalArgumentException("TextureRegionMobile.from: rect must hold 2, 4 or 6 numbers")
    }
    return TextureRegionMobile(
        originX = origin[0], originY = origin[1], originZ = origin[2],
        sizeWidth = size[0], sizeHeight = size[1], sizeDepth = size[2],
        bytesPerRow = null, rowsPerImage = null,
    )
}

/**
 * Set `bytesPerRow` for source data with row padding.
 * It must be at least the region width times the bytes per pixel.
 * When unset, rows are tightly packed unless the data is exactly the size
 * of rows padded to 256 bytes, in which case they read at the padded stride.
 */
fun TextureRegionMobile.withStride(bytesPerRow: Int): TextureRegionMobile = copy(bytesPerRow = bytesPerRow.toUInt())

fun TextureRegionMobile.withStride(bytesPerRow: UInt): TextureRegionMobile = copy(bytesPerRow = bytesPerRow)

/** Set `rowsPerImage`, which defaults to the region height, for layered or 3D uploads. */
fun TextureRegionMobile.withRows(rowsPerImage: Int): TextureRegionMobile = copy(rowsPerImage = rowsPerImage.toUInt())

fun TextureRegionMobile.withRows(rowsPerImage: UInt): TextureRegionMobile = copy(rowsPerImage = rowsPerImage)
