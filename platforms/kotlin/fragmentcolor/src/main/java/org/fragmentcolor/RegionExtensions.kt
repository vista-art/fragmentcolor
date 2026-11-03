package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated TextureRegion
// record. Rust builds a TextureRegion through from([x, y, w, h]), with_stride
// and with_rows. The uniffi record is plain fields, so these functions
// reproduce the Rust call shape, as Region+Extensions.swift does on Swift.

@Deprecated("Use TextureRegion", ReplaceWith("TextureRegion"))
typealias TextureRegionMobile = TextureRegion

/**
 * A texture region from `[width, height]`, `[x, y, width, height]`
 * or `[x, y, z, width, height, depth]`, as Rust's `TextureRegion::from` reads them.
 * Throws [FragmentColorException] for any other length.
 */
fun TextureRegion.Companion.from(rect: List<UInt>): TextureRegion {
    val (origin, size) = when (rect.size) {
        2 -> listOf(0u, 0u, 0u) to listOf(rect[0], rect[1], 1u)
        4 -> listOf(rect[0], rect[1], 0u) to listOf(rect[2], rect[3], 1u)
        6 -> rect.subList(0, 3) to rect.subList(3, 6)
        else -> throw FragmentColorException.Render("TextureRegion.from: rect must hold 2, 4 or 6 numbers")
    }
    return TextureRegion(
        originX = origin[0], originY = origin[1], originZ = origin[2],
        sizeWidth = size[0], sizeHeight = size[1], sizeDepth = size[2],
        bytesPerRow = null, rowsPerImage = null,
    )
}

/**
 * The pixels a [ScreenRegion] covers, at depth 0.
 * Throws [FragmentColorException] for a region whose max isn't past its min, as Swift does.
 */
fun TextureRegion.Companion.from(region: ScreenRegion): TextureRegion {
    if (region.maxX <= region.minX || region.maxY <= region.minY) {
        throw FragmentColorException.Render(
            "TextureRegion.from: a region needs each max past its min, got x from ${region.minX} to ${region.maxX} and y from ${region.minY} to ${region.maxY}",
        )
    }
    return TextureRegion(
        originX = region.minX, originY = region.minY, originZ = 0u,
        sizeWidth = region.maxX - region.minX,
        sizeHeight = region.maxY - region.minY,
        sizeDepth = 1u,
        bytesPerRow = null, rowsPerImage = null,
    )
}

/**
 * Set `bytesPerRow` for source data with row padding.
 * It must be at least the region width times the bytes per pixel.
 * When unset, rows are tightly packed unless the data is exactly the size
 * of rows padded to 256 bytes, in which case they read at the padded stride.
 */
fun TextureRegion.withStride(bytesPerRow: UInt): TextureRegion = copy(bytesPerRow = bytesPerRow)

/** Set `rowsPerImage`, which defaults to the region height, for layered or 3D uploads. */
fun TextureRegion.withRows(rowsPerImage: UInt): TextureRegion = copy(rowsPerImage = rowsPerImage)
