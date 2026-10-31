package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated Material API,
// matching Material+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well.

fun Material.metallic(value: Double): Material = metallic(value.toFloat())

fun Material.roughness(value: Double): Material = roughness(value.toFloat())

fun Material.normalScale(value: Double): Material = normalScale(value.toFloat())

fun Material.occlusionStrength(value: Double): Material = occlusionStrength(value.toFloat())

fun Material.alphaCutoff(value: Double): Material = alphaCutoff(value.toFloat())

fun Material.baseColor(color: FloatArray): Material = baseColor(color.toList())

@JvmName("baseColorDoubles")
fun Material.baseColor(color: List<Double>): Material = baseColor(readFloats(color, "Material.baseColor"))

fun Material.baseColor(color: DoubleArray): Material = baseColor(readFloats(color, "Material.baseColor"))

fun Material.emissive(factor: FloatArray): Material = emissive(factor.toList())

@JvmName("emissiveDoubles")
fun Material.emissive(factor: List<Double>): Material = emissive(readFloats(factor, "Material.emissive"))

fun Material.emissive(factor: DoubleArray): Material = emissive(readFloats(factor, "Material.emissive"))

fun Material.uvTransform(offset: FloatArray, scale: FloatArray, rotation: Float): Material =
    uvTransform(offset.toList(), scale.toList(), rotation)

@JvmName("uvTransformDoubles")
fun Material.uvTransform(offset: List<Double>, scale: List<Double>, rotation: Double): Material =
    uvTransform(
        readFloats(offset, "Material.uvTransform: offset"),
        readFloats(scale, "Material.uvTransform: scale"),
        rotation.toFloat(),
    )

fun Material.uvTransform(offset: DoubleArray, scale: DoubleArray, rotation: Double): Material =
    uvTransform(
        readFloats(offset, "Material.uvTransform: offset"),
        readFloats(scale, "Material.uvTransform: scale"),
        rotation.toFloat(),
    )

fun Material.textureTransform(slot: TextureSlot, offset: FloatArray, scale: FloatArray, rotation: Float): Material =
    textureTransform(slot, offset.toList(), scale.toList(), rotation)

@JvmName("textureTransformDoubles")
fun Material.textureTransform(slot: TextureSlot, offset: List<Double>, scale: List<Double>, rotation: Double): Material =
    textureTransform(
        slot,
        readFloats(offset, "Material.textureTransform: offset"),
        readFloats(scale, "Material.textureTransform: scale"),
        rotation.toFloat(),
    )

fun Material.textureTransform(slot: TextureSlot, offset: DoubleArray, scale: DoubleArray, rotation: Double): Material =
    textureTransform(
        slot,
        readFloats(offset, "Material.textureTransform: offset"),
        readFloats(scale, "Material.textureTransform: scale"),
        rotation.toFloat(),
    )

// Each texture setter takes what `Renderer.createTexture` takes, spelled the same way.
// A Texture binds at once, and every other input uploads when the scene loads, as in Rust.

fun Material.baseColorTexture(bytes: ByteArray, usage: TextureUsage? = null): Material =
    baseColorSource(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

fun Material.baseColorTexture(bytes: List<Int>, usage: TextureUsage? = null): Material =
    baseColorTexture(byteArray(bytes, "Material.baseColorTexture"), usage)

fun Material.baseColorTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Material =
    baseColorSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

fun Material.baseColorTexture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): Material =
    baseColorTexture(byteArray(bytes, "Material.baseColorTexture"), format, usage)

fun Material.baseColorTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Material =
    baseColorSource(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

fun Material.baseColorTexture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): Material =
    baseColorTexture(byteArray(bytes, "Material.baseColorTexture"), size, usage)

fun Material.baseColorTexture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): Material =
    baseColorTexture(bytes, sizeRecord(size, "Material.baseColorTexture"), usage)

fun Material.baseColorTexture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): Material =
    baseColorTexture(byteArray(bytes, "Material.baseColorTexture"), size, usage)

fun Material.baseColorTexture(bytes: ByteArray, options: TextureOptions): Material =
    baseColorSource(TextureInputMobile.Bytes(bytes), options)

fun Material.baseColorTexture(bytes: List<Int>, options: TextureOptions): Material =
    baseColorTexture(byteArray(bytes, "Material.baseColorTexture"), options)

fun Material.baseColorTexture(path: String): Material = baseColorSource(pathInput(path))

fun Material.baseColorTexture(path: String, format: TextureFormat): Material =
    baseColorSource(pathInput(path), textureOptions(format = format))

fun Material.baseColorTexture(path: String, options: TextureOptions): Material = baseColorSource(pathInput(path), options)

fun Material.baseColorTexture(chain: Mipmap): Material = baseColorSource(TextureInputMobile.Prepared(chain))

fun Material.baseColorTexture(chain: Mipmap, options: TextureOptions): Material =
    baseColorSource(TextureInputMobile.Prepared(chain), options)

fun Material.metallicRoughnessTexture(bytes: ByteArray, usage: TextureUsage? = null): Material =
    metallicRoughnessSource(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

fun Material.metallicRoughnessTexture(bytes: List<Int>, usage: TextureUsage? = null): Material =
    metallicRoughnessTexture(byteArray(bytes, "Material.metallicRoughnessTexture"), usage)

fun Material.metallicRoughnessTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Material =
    metallicRoughnessSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

fun Material.metallicRoughnessTexture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): Material =
    metallicRoughnessTexture(byteArray(bytes, "Material.metallicRoughnessTexture"), format, usage)

fun Material.metallicRoughnessTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Material =
    metallicRoughnessSource(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

fun Material.metallicRoughnessTexture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): Material =
    metallicRoughnessTexture(byteArray(bytes, "Material.metallicRoughnessTexture"), size, usage)

fun Material.metallicRoughnessTexture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): Material =
    metallicRoughnessTexture(bytes, sizeRecord(size, "Material.metallicRoughnessTexture"), usage)

fun Material.metallicRoughnessTexture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): Material =
    metallicRoughnessTexture(byteArray(bytes, "Material.metallicRoughnessTexture"), size, usage)

fun Material.metallicRoughnessTexture(bytes: ByteArray, options: TextureOptions): Material =
    metallicRoughnessSource(TextureInputMobile.Bytes(bytes), options)

fun Material.metallicRoughnessTexture(bytes: List<Int>, options: TextureOptions): Material =
    metallicRoughnessTexture(byteArray(bytes, "Material.metallicRoughnessTexture"), options)

fun Material.metallicRoughnessTexture(path: String): Material = metallicRoughnessSource(pathInput(path))

fun Material.metallicRoughnessTexture(path: String, format: TextureFormat): Material =
    metallicRoughnessSource(pathInput(path), textureOptions(format = format))

fun Material.metallicRoughnessTexture(path: String, options: TextureOptions): Material = metallicRoughnessSource(pathInput(path), options)

fun Material.metallicRoughnessTexture(chain: Mipmap): Material = metallicRoughnessSource(TextureInputMobile.Prepared(chain))

fun Material.metallicRoughnessTexture(chain: Mipmap, options: TextureOptions): Material =
    metallicRoughnessSource(TextureInputMobile.Prepared(chain), options)

fun Material.normalTexture(bytes: ByteArray, usage: TextureUsage? = null): Material =
    normalSource(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

fun Material.normalTexture(bytes: List<Int>, usage: TextureUsage? = null): Material =
    normalTexture(byteArray(bytes, "Material.normalTexture"), usage)

fun Material.normalTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Material =
    normalSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

fun Material.normalTexture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): Material =
    normalTexture(byteArray(bytes, "Material.normalTexture"), format, usage)

fun Material.normalTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Material =
    normalSource(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

fun Material.normalTexture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): Material =
    normalTexture(byteArray(bytes, "Material.normalTexture"), size, usage)

fun Material.normalTexture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): Material =
    normalTexture(bytes, sizeRecord(size, "Material.normalTexture"), usage)

fun Material.normalTexture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): Material =
    normalTexture(byteArray(bytes, "Material.normalTexture"), size, usage)

fun Material.normalTexture(bytes: ByteArray, options: TextureOptions): Material =
    normalSource(TextureInputMobile.Bytes(bytes), options)

fun Material.normalTexture(bytes: List<Int>, options: TextureOptions): Material =
    normalTexture(byteArray(bytes, "Material.normalTexture"), options)

fun Material.normalTexture(path: String): Material = normalSource(pathInput(path))

fun Material.normalTexture(path: String, format: TextureFormat): Material =
    normalSource(pathInput(path), textureOptions(format = format))

fun Material.normalTexture(path: String, options: TextureOptions): Material = normalSource(pathInput(path), options)

fun Material.normalTexture(chain: Mipmap): Material = normalSource(TextureInputMobile.Prepared(chain))

fun Material.normalTexture(chain: Mipmap, options: TextureOptions): Material =
    normalSource(TextureInputMobile.Prepared(chain), options)

fun Material.occlusionTexture(bytes: ByteArray, usage: TextureUsage? = null): Material =
    occlusionSource(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

fun Material.occlusionTexture(bytes: List<Int>, usage: TextureUsage? = null): Material =
    occlusionTexture(byteArray(bytes, "Material.occlusionTexture"), usage)

fun Material.occlusionTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Material =
    occlusionSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

fun Material.occlusionTexture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): Material =
    occlusionTexture(byteArray(bytes, "Material.occlusionTexture"), format, usage)

fun Material.occlusionTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Material =
    occlusionSource(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

fun Material.occlusionTexture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): Material =
    occlusionTexture(byteArray(bytes, "Material.occlusionTexture"), size, usage)

fun Material.occlusionTexture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): Material =
    occlusionTexture(bytes, sizeRecord(size, "Material.occlusionTexture"), usage)

fun Material.occlusionTexture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): Material =
    occlusionTexture(byteArray(bytes, "Material.occlusionTexture"), size, usage)

fun Material.occlusionTexture(bytes: ByteArray, options: TextureOptions): Material =
    occlusionSource(TextureInputMobile.Bytes(bytes), options)

fun Material.occlusionTexture(bytes: List<Int>, options: TextureOptions): Material =
    occlusionTexture(byteArray(bytes, "Material.occlusionTexture"), options)

fun Material.occlusionTexture(path: String): Material = occlusionSource(pathInput(path))

fun Material.occlusionTexture(path: String, format: TextureFormat): Material =
    occlusionSource(pathInput(path), textureOptions(format = format))

fun Material.occlusionTexture(path: String, options: TextureOptions): Material = occlusionSource(pathInput(path), options)

fun Material.occlusionTexture(chain: Mipmap): Material = occlusionSource(TextureInputMobile.Prepared(chain))

fun Material.occlusionTexture(chain: Mipmap, options: TextureOptions): Material =
    occlusionSource(TextureInputMobile.Prepared(chain), options)

fun Material.emissiveTexture(bytes: ByteArray, usage: TextureUsage? = null): Material =
    emissiveSource(TextureInputMobile.Bytes(bytes), textureOptions(usage = usage))

fun Material.emissiveTexture(bytes: List<Int>, usage: TextureUsage? = null): Material =
    emissiveTexture(byteArray(bytes, "Material.emissiveTexture"), usage)

fun Material.emissiveTexture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): Material =
    emissiveSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format, usage = usage))

fun Material.emissiveTexture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): Material =
    emissiveTexture(byteArray(bytes, "Material.emissiveTexture"), format, usage)

fun Material.emissiveTexture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): Material =
    emissiveSource(TextureInputMobile.Bytes(bytes), textureOptions(size = size, usage = usage))

fun Material.emissiveTexture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): Material =
    emissiveTexture(byteArray(bytes, "Material.emissiveTexture"), size, usage)

fun Material.emissiveTexture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): Material =
    emissiveTexture(bytes, sizeRecord(size, "Material.emissiveTexture"), usage)

fun Material.emissiveTexture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): Material =
    emissiveTexture(byteArray(bytes, "Material.emissiveTexture"), size, usage)

fun Material.emissiveTexture(bytes: ByteArray, options: TextureOptions): Material =
    emissiveSource(TextureInputMobile.Bytes(bytes), options)

fun Material.emissiveTexture(bytes: List<Int>, options: TextureOptions): Material =
    emissiveTexture(byteArray(bytes, "Material.emissiveTexture"), options)

fun Material.emissiveTexture(path: String): Material = emissiveSource(pathInput(path))

fun Material.emissiveTexture(path: String, format: TextureFormat): Material =
    emissiveSource(pathInput(path), textureOptions(format = format))

fun Material.emissiveTexture(path: String, options: TextureOptions): Material = emissiveSource(pathInput(path), options)

fun Material.emissiveTexture(chain: Mipmap): Material = emissiveSource(TextureInputMobile.Prepared(chain))

fun Material.emissiveTexture(chain: Mipmap, options: TextureOptions): Material =
    emissiveSource(TextureInputMobile.Prepared(chain), options)
