package org.fragmentcolor

// Idiomatic Kotlin wrappers on top of the uniffi-generated material API,
// matching Material+Extensions.swift. The generated calls take Float and
// List<Float>; these take Double, FloatArray and DoubleArray as well.
// A number setter throws for NaN, an infinity or a Double past the Float range.

fun PbrMaterial.metallic(value: Double): PbrMaterial = metallic(readFinite(value, "PbrMaterial.metallic: value"))

fun PbrMaterial.roughness(value: Double): PbrMaterial = roughness(readFinite(value, "PbrMaterial.roughness: value"))

fun PbrMaterial.normalScale(value: Double): PbrMaterial = normalScale(readFinite(value, "PbrMaterial.normalScale: value"))

fun PbrMaterial.occlusionStrength(value: Double): PbrMaterial =
    occlusionStrength(readFinite(value, "PbrMaterial.occlusionStrength: value"))

fun PbrMaterial.alphaCutoff(value: Double): PbrMaterial = alphaCutoff(readFinite(value, "PbrMaterial.alphaCutoff: value"))

fun PbrMaterial.baseColor(color: FloatArray): PbrMaterial = baseColor(color.toList())

@JvmName("baseColorDoubles")
fun PbrMaterial.baseColor(color: List<Double>): PbrMaterial = baseColor(readFloats(color, "PbrMaterial.baseColor"))

fun PbrMaterial.baseColor(color: DoubleArray): PbrMaterial = baseColor(readFloats(color, "PbrMaterial.baseColor"))

fun PbrMaterial.emissive(factor: FloatArray): PbrMaterial = emissive(factor.toList())

@JvmName("emissiveDoubles")
fun PbrMaterial.emissive(factor: List<Double>): PbrMaterial = emissive(readFloats(factor, "PbrMaterial.emissive"))

fun PbrMaterial.emissive(factor: DoubleArray): PbrMaterial = emissive(readFloats(factor, "PbrMaterial.emissive"))

fun PbrMaterial.uvTransform(offset: FloatArray, scale: FloatArray, rotation: Float): PbrMaterial =
    uvTransform(offset.toList(), scale.toList(), rotation)

@JvmName("uvTransformDoubles")
fun PbrMaterial.uvTransform(offset: List<Double>, scale: List<Double>, rotation: Double): PbrMaterial =
    uvTransform(
        readFloats(offset, "PbrMaterial.uvTransform: offset"),
        readFloats(scale, "PbrMaterial.uvTransform: scale"),
        readFinite(rotation, "PbrMaterial.uvTransform: rotation"),
    )

fun PbrMaterial.uvTransform(offset: DoubleArray, scale: DoubleArray, rotation: Double): PbrMaterial =
    uvTransform(
        readFloats(offset, "PbrMaterial.uvTransform: offset"),
        readFloats(scale, "PbrMaterial.uvTransform: scale"),
        readFinite(rotation, "PbrMaterial.uvTransform: rotation"),
    )

fun UnlitMaterial.alphaCutoff(value: Double): UnlitMaterial =
    alphaCutoff(readFinite(value, "UnlitMaterial.alphaCutoff: value"))

fun UnlitMaterial.baseColor(color: FloatArray): UnlitMaterial = baseColor(color.toList())

@JvmName("baseColorDoubles")
fun UnlitMaterial.baseColor(color: List<Double>): UnlitMaterial = baseColor(readFloats(color, "UnlitMaterial.baseColor"))

fun UnlitMaterial.baseColor(color: DoubleArray): UnlitMaterial = baseColor(readFloats(color, "UnlitMaterial.baseColor"))

fun ToonMaterial.alphaCutoff(value: Double): ToonMaterial =
    alphaCutoff(readFinite(value, "ToonMaterial.alphaCutoff: value"))

fun ToonMaterial.highlightSize(value: Double): ToonMaterial =
    highlightSize(readFinite(value, "ToonMaterial.highlightSize: value"))

fun ToonMaterial.rimWidth(value: Double): ToonMaterial = rimWidth(readFinite(value, "ToonMaterial.rimWidth: value"))

fun ToonMaterial.baseColor(color: FloatArray): ToonMaterial = baseColor(color.toList())

@JvmName("baseColorDoubles")
fun ToonMaterial.baseColor(color: List<Double>): ToonMaterial = baseColor(readFloats(color, "ToonMaterial.baseColor"))

fun ToonMaterial.baseColor(color: DoubleArray): ToonMaterial = baseColor(readFloats(color, "ToonMaterial.baseColor"))

fun ToonMaterial.highlight(color: FloatArray): ToonMaterial = highlight(color.toList())

@JvmName("highlightDoubles")
fun ToonMaterial.highlight(color: List<Double>): ToonMaterial = highlight(readFloats(color, "ToonMaterial.highlight"))

fun ToonMaterial.highlight(color: DoubleArray): ToonMaterial = highlight(readFloats(color, "ToonMaterial.highlight"))

fun ToonMaterial.rim(color: FloatArray): ToonMaterial = rim(color.toList())

@JvmName("rimDoubles")
fun ToonMaterial.rim(color: List<Double>): ToonMaterial = rim(readFloats(color, "ToonMaterial.rim"))

fun ToonMaterial.rim(color: DoubleArray): ToonMaterial = rim(readFloats(color, "ToonMaterial.rim"))

fun MaterialMap.transform(offset: FloatArray, scale: FloatArray, rotation: Float): MaterialMap =
    transform(offset.toList(), scale.toList(), rotation)

@JvmName("transformDoubles")
fun MaterialMap.transform(offset: List<Double>, scale: List<Double>, rotation: Double): MaterialMap =
    transform(
        readFloats(offset, "MaterialMap.transform: offset"),
        readFloats(scale, "MaterialMap.transform: scale"),
        readFinite(rotation, "MaterialMap.transform: rotation"),
    )

fun MaterialMap.transform(offset: DoubleArray, scale: DoubleArray, rotation: Double): MaterialMap =
    transform(
        readFloats(offset, "MaterialMap.transform: offset"),
        readFloats(scale, "MaterialMap.transform: scale"),
        readFinite(rotation, "MaterialMap.transform: rotation"),
    )

// `texture` takes what `Renderer.createTexture` takes, spelled the same way, and another map.
// A Texture binds at once, another map shares its texture, and every other input uploads
// when the scene loads, as in Rust. The generated `texture(texture)` takes the Texture,
// `texture(map)` reaches `share`, and the rest reach `source`.

fun MaterialMap.texture(map: MaterialMap): MaterialMap = share(map)

fun MaterialMap.texture(bytes: ByteArray, usage: TextureUsage? = null): MaterialMap =
    source(TextureData.Bytes(bytes), textureOptions(usage = usage))

fun MaterialMap.texture(bytes: List<Int>, usage: TextureUsage? = null): MaterialMap =
    texture(byteArray(bytes, "MaterialMap.texture"), usage)

fun MaterialMap.texture(bytes: ByteArray, format: TextureFormat, usage: TextureUsage? = null): MaterialMap =
    source(TextureData.Bytes(bytes), textureOptions(format = format, usage = usage))

fun MaterialMap.texture(bytes: List<Int>, format: TextureFormat, usage: TextureUsage? = null): MaterialMap =
    texture(byteArray(bytes, "MaterialMap.texture"), format, usage)

fun MaterialMap.texture(bytes: ByteArray, size: Size, usage: TextureUsage? = null): MaterialMap =
    source(TextureData.Bytes(bytes), textureOptions(size = size, usage = usage))

fun MaterialMap.texture(bytes: List<Int>, size: Size, usage: TextureUsage? = null): MaterialMap =
    texture(byteArray(bytes, "MaterialMap.texture"), size, usage)

fun MaterialMap.texture(bytes: ByteArray, size: List<UInt>, usage: TextureUsage? = null): MaterialMap =
    texture(bytes, sizeRecord(size, "MaterialMap.texture"), usage)

fun MaterialMap.texture(bytes: List<Int>, size: List<UInt>, usage: TextureUsage? = null): MaterialMap =
    texture(byteArray(bytes, "MaterialMap.texture"), size, usage)

fun MaterialMap.texture(bytes: ByteArray, options: TextureOptions): MaterialMap =
    source(TextureData.Bytes(bytes), options)

fun MaterialMap.texture(bytes: List<Int>, options: TextureOptions): MaterialMap =
    texture(byteArray(bytes, "MaterialMap.texture"), options)

fun MaterialMap.texture(path: String): MaterialMap = source(pathInput(path))

fun MaterialMap.texture(path: String, format: TextureFormat): MaterialMap =
    source(pathInput(path), textureOptions(format = format))

fun MaterialMap.texture(path: String, options: TextureOptions): MaterialMap = source(pathInput(path), options)

fun MaterialMap.texture(chain: Mipmap): MaterialMap = source(TextureData.Prepared(chain))

fun MaterialMap.texture(chain: Mipmap, options: TextureOptions): MaterialMap =
    source(TextureData.Prepared(chain), options)
