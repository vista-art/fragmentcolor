package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Texture API,
// matching the call shapes of Texture+Extensions.swift.

@Deprecated("Use TextureTarget", ReplaceWith("TextureTarget"))
typealias MobileTextureTarget = TextureTarget

@Deprecated("Use WindowTarget", ReplaceWith("WindowTarget"))
typealias MobileWindowTarget = WindowTarget

// ── Texture ──────────────────────────────────────────────────────────────────

/** Upload pixels, one [Int] per byte, into the whole texture. */
fun Texture.write(bytes: List<Int>) {
    write(byteArray(bytes, "write"))
}

/**
 * Upload pixels into part of the texture, given as `[width, height]`,
 * `[x, y, width, height]` or `[x, y, z, width, height, depth]`.
 */
fun Texture.writeRegion(bytes: ByteArray, rect: List<UInt>) {
    writeRegion(bytes, TextureRegion.from(rect))
}

/** Upload pixels, one [Int] per byte, into part of the texture given as a list of numbers. */
@JvmName("writeRegionInts")
fun Texture.writeRegion(bytes: List<Int>, rect: List<UInt>) {
    writeRegion(byteArray(bytes, "writeRegion"), rect)
}

/** Upload pixels, one [Int] per byte, into a [TextureRegion]. */
fun Texture.writeRegion(bytes: List<Int>, region: TextureRegion) {
    writeRegion(byteArray(bytes, "writeRegion"), region)
}

/** Upload pixels into the pixels a [ScreenRegion] covers. */
fun Texture.writeRegion(bytes: ByteArray, region: ScreenRegion) {
    writeRegion(bytes, TextureRegion.from(region))
}

/** Upload pixels, one [Int] per byte, into the pixels a [ScreenRegion] covers. */
fun Texture.writeRegion(bytes: List<Int>, region: ScreenRegion) {
    writeRegion(byteArray(bytes, "writeRegion"), region)
}

/** Resize the texture from a `[width, height]` list. */
fun Texture.resize(size: List<UInt>) {
    resize(sizeRecord(size, "resize"))
}

// ── Targets ──────────────────────────────────────────────────────────────────

/** Resize the target from a `[width, height]` list. */
fun TextureTarget.resize(size: List<UInt>) {
    resize(sizeRecord(size, "resize"))
}

/** Resize the target from a `[width, height]` list. */
fun WindowTarget.resize(size: List<UInt>) {
    resize(sizeRecord(size, "resize"))
}

// ── Mipmap ───────────────────────────────────────────────────────────────────

/** Build a chain from raw pixels of [format] and their `[width, height]`. */
fun Mipmap.Companion.build(bytes: ByteArray, format: TextureFormat, size: List<UInt>): Mipmap =
    build(bytes, format, sizeRecord(size, "Mipmap.build"))

// ── TextureUsage ─────────────────────────────────────────────────────────────

/** The texture can be the source of a copy, such as `Renderer.readTexture`. */
val TextureUsage.Companion.COPY_SRC: TextureUsage get() = TextureUsage(1u shl 0)

/** The texture can be the destination of a copy, such as `Texture.write`. */
val TextureUsage.Companion.COPY_DST: TextureUsage get() = TextureUsage(1u shl 1)

/** A shader can sample the texture or load from it. */
val TextureUsage.Companion.TEXTURE_BINDING: TextureUsage get() = TextureUsage(1u shl 2)

/** A shader can write the texture as a storage texture. */
val TextureUsage.Companion.STORAGE_BINDING: TextureUsage get() = TextureUsage(1u shl 3)

/** A pass can render into the texture. */
val TextureUsage.Companion.RENDER_ATTACHMENT: TextureUsage get() = TextureUsage(1u shl 4)

/** Both sets of flags, as Rust's `|` combines them. */
infix fun TextureUsage.or(other: TextureUsage): TextureUsage = TextureUsage(bits or other.bits)
