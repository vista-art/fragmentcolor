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
fun Material.baseColor(color: List<Double>): Material = baseColor(color.map { it.toFloat() })

fun Material.baseColor(color: DoubleArray): Material = baseColor(color.map { it.toFloat() })

fun Material.emissive(factor: FloatArray): Material = emissive(factor.toList())

@JvmName("emissiveDoubles")
fun Material.emissive(factor: List<Double>): Material = emissive(factor.map { it.toFloat() })

fun Material.emissive(factor: DoubleArray): Material = emissive(factor.map { it.toFloat() })

fun Material.uvTransform(offset: FloatArray, scale: FloatArray, rotation: Float): Material =
    uvTransform(offset.toList(), scale.toList(), rotation)

@JvmName("uvTransformDoubles")
fun Material.uvTransform(offset: List<Double>, scale: List<Double>, rotation: Double): Material =
    uvTransform(offset.map { it.toFloat() }, scale.map { it.toFloat() }, rotation.toFloat())

fun Material.uvTransform(offset: DoubleArray, scale: DoubleArray, rotation: Double): Material =
    uvTransform(offset.map { it.toFloat() }, scale.map { it.toFloat() }, rotation.toFloat())

fun Material.textureTransform(slot: TextureSlot, offset: FloatArray, scale: FloatArray, rotation: Float): Material =
    textureTransform(slot, offset.toList(), scale.toList(), rotation)

@JvmName("textureTransformDoubles")
fun Material.textureTransform(slot: TextureSlot, offset: List<Double>, scale: List<Double>, rotation: Double): Material =
    textureTransform(slot, offset.map { it.toFloat() }, scale.map { it.toFloat() }, rotation.toFloat())

fun Material.textureTransform(slot: TextureSlot, offset: DoubleArray, scale: DoubleArray, rotation: Double): Material =
    textureTransform(slot, offset.map { it.toFloat() }, scale.map { it.toFloat() }, rotation.toFloat())

// Each texture setter takes what `Renderer.createTexture` takes: a Texture, or
// encoded bytes, a file path or http(s) URL, or a Mipmap. Inputs other than a
// Texture upload when the scene loads, as in Rust.

fun Material.baseColorTexture(bytes: ByteArray): Material = baseColorSource(TextureInputMobile.Bytes(bytes))

fun Material.baseColorTexture(bytes: ByteArray, format: TextureFormat): Material =
    baseColorSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format))

fun Material.baseColorTexture(path: String): Material = baseColorSource(pathInput(path))

fun Material.baseColorTexture(chain: Mipmap): Material = baseColorSource(TextureInputMobile.Prepared(chain))

fun Material.metallicRoughnessTexture(bytes: ByteArray): Material = metallicRoughnessSource(TextureInputMobile.Bytes(bytes))

fun Material.metallicRoughnessTexture(bytes: ByteArray, format: TextureFormat): Material =
    metallicRoughnessSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format))

fun Material.metallicRoughnessTexture(path: String): Material = metallicRoughnessSource(pathInput(path))

fun Material.metallicRoughnessTexture(chain: Mipmap): Material = metallicRoughnessSource(TextureInputMobile.Prepared(chain))

fun Material.normalTexture(bytes: ByteArray): Material = normalSource(TextureInputMobile.Bytes(bytes))

fun Material.normalTexture(bytes: ByteArray, format: TextureFormat): Material =
    normalSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format))

fun Material.normalTexture(path: String): Material = normalSource(pathInput(path))

fun Material.normalTexture(chain: Mipmap): Material = normalSource(TextureInputMobile.Prepared(chain))

fun Material.occlusionTexture(bytes: ByteArray): Material = occlusionSource(TextureInputMobile.Bytes(bytes))

fun Material.occlusionTexture(bytes: ByteArray, format: TextureFormat): Material =
    occlusionSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format))

fun Material.occlusionTexture(path: String): Material = occlusionSource(pathInput(path))

fun Material.occlusionTexture(chain: Mipmap): Material = occlusionSource(TextureInputMobile.Prepared(chain))

fun Material.emissiveTexture(bytes: ByteArray): Material = emissiveSource(TextureInputMobile.Bytes(bytes))

fun Material.emissiveTexture(bytes: ByteArray, format: TextureFormat): Material =
    emissiveSource(TextureInputMobile.Bytes(bytes), textureOptions(format = format))

fun Material.emissiveTexture(path: String): Material = emissiveSource(pathInput(path))

fun Material.emissiveTexture(chain: Mipmap): Material = emissiveSource(TextureInputMobile.Prepared(chain))
