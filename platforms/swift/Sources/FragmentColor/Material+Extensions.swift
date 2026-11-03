//
//  Material+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated Material API.
//  Mirrors the spelling used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

extension Material {
    // MARK: - `Material.custom(shader)` — drop the `shader:` label.

    public static func custom(_ shader: Shader) -> Material {
        return Material.custom(shader: shader)
    }

    // MARK: - Setters that take a single scalar Float.
    //
    // `metallic(value:)`, `roughness(value:)`, `normalScale(value:)`,
    // `occlusionStrength(value:)`, `alphaCutoff(value:)`, `doubleSided(value:)`
    // — accept positional `Float` / `Double` / `Bool` so example code can
    // write `mat.metallic(0.5)` instead of `mat.metallic(value: 0.5)`.
    // A number setter throws for NaN, an infinity or a `Double` past the `Float` range.

    @discardableResult
    public func metallic(_ value: Float) throws -> Material {
        return try metallic(value: value)
    }
    @discardableResult
    public func metallic(_ value: Double) throws -> Material {
        return try metallic(value: readFinite(value, "Material.metallic: value"))
    }

    @discardableResult
    public func roughness(_ value: Float) throws -> Material {
        return try roughness(value: value)
    }
    @discardableResult
    public func roughness(_ value: Double) throws -> Material {
        return try roughness(value: readFinite(value, "Material.roughness: value"))
    }

    @discardableResult
    public func normalScale(_ value: Float) throws -> Material {
        return try normalScale(value: value)
    }
    @discardableResult
    public func normalScale(_ value: Double) throws -> Material {
        return try normalScale(value: readFinite(value, "Material.normalScale: value"))
    }

    @discardableResult
    public func occlusionStrength(_ value: Float) throws -> Material {
        return try occlusionStrength(value: value)
    }
    @discardableResult
    public func occlusionStrength(_ value: Double) throws -> Material {
        return try occlusionStrength(value: readFinite(value, "Material.occlusionStrength: value"))
    }

    @discardableResult
    public func alphaCutoff(_ value: Float) throws -> Material {
        return try alphaCutoff(value: value)
    }
    @discardableResult
    public func alphaCutoff(_ value: Double) throws -> Material {
        return try alphaCutoff(value: readFinite(value, "Material.alphaCutoff: value"))
    }

    @discardableResult
    public func doubleSided(_ value: Bool) -> Material {
        return doubleSided(value: value)
    }

    // MARK: - Setters that take a Float vector or Bool.

    @discardableResult
    public func baseColor(_ color: [Float]) throws -> Material {
        return try baseColor(color: color)
    }
    @discardableResult
    @_disfavoredOverload
    public func baseColor(_ color: [Double]) throws -> Material {
        return try baseColor(color: readFloats(color, "Material.baseColor"))
    }

    @discardableResult
    public func emissive(_ factor: [Float]) throws -> Material {
        return try emissive(factor: factor)
    }
    @discardableResult
    @_disfavoredOverload
    public func emissive(_ factor: [Double]) throws -> Material {
        return try emissive(factor: readFloats(factor, "Material.emissive"))
    }

    @discardableResult
    public func alphaMode(_ mode: AlphaMode) -> Material {
        return alphaMode(mode: mode)
    }

    @discardableResult
    public func uvTransform(_ offset: [Float], _ scale: [Float], _ rotation: Float) throws -> Material {
        return try uvTransform(offset: offset, scale: scale, rotation: rotation)
    }
    @discardableResult
    @_disfavoredOverload
    public func uvTransform(_ offset: [Double], _ scale: [Double], _ rotation: Double) throws -> Material {
        return try uvTransform(
            offset: readFloats(offset, "Material.uvTransform: offset"),
            scale: readFloats(scale, "Material.uvTransform: scale"),
            rotation: readFinite(rotation, "Material.uvTransform: rotation")
        )
    }

    @discardableResult
    public func textureUv(_ slot: TextureSlot, _ set: UInt32) throws -> Material {
        return try textureUv(slot: slot, set: set)
    }

    @discardableResult
    public func textureTransform(
        _ slot: TextureSlot, _ offset: [Float], _ scale: [Float], _ rotation: Float
    ) throws -> Material {
        return try textureTransform(slot: slot, offset: offset, scale: scale, rotation: rotation)
    }
    @discardableResult
    @_disfavoredOverload
    public func textureTransform(
        _ slot: TextureSlot, _ offset: [Double], _ scale: [Double], _ rotation: Double
    ) throws -> Material {
        return try textureTransform(
            slot: slot,
            offset: readFloats(offset, "Material.textureTransform: offset"),
            scale: readFloats(scale, "Material.textureTransform: scale"),
            rotation: readFinite(rotation, "Material.textureTransform: rotation")
        )
    }

    // MARK: - Texture setters

    // Each texture setter takes what `Renderer.createTexture` takes, spelled the same way.
    // A `Texture` binds at once, and every other input uploads when the scene loads, as in Rust.
    // A setter throws for an input it can't use, such as a slot a custom shader declares as a number.

    @discardableResult
    public func baseColorTexture(_ texture: Texture) throws -> Material {
        try baseColorTexture(texture: texture)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, usage: TextureUsage? = nil) throws -> Material {
        try baseColorSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(try byteData(bytes, "Material.baseColorTexture"), usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try baseColorSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(try byteData(bytes, "Material.baseColorTexture"), format, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try baseColorSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(try byteData(bytes, "Material.baseColorTexture"), size, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(bytes, try sizeRecord(size, "Material.baseColorTexture"), usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try baseColorTexture(try byteData(bytes, "Material.baseColorTexture"), size, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ options: TextureOptions) throws -> Material {
        try baseColorSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ options: TextureOptions) throws -> Material {
        try baseColorTexture(Data(bytes), options)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [Int], _ options: TextureOptions) throws -> Material {
        try baseColorTexture(try byteData(bytes, "Material.baseColorTexture"), options)
    }

    @discardableResult
    public func baseColorTexture(_ path: String) throws -> Material {
        try baseColorSource(input: pathInput(path))
    }

    @discardableResult
    public func baseColorTexture(_ path: String, _ format: TextureFormat) throws -> Material {
        try baseColorSource(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func baseColorTexture(_ path: String, _ options: TextureOptions) throws -> Material {
        try baseColorSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func baseColorTexture(_ chain: Mipmap) throws -> Material {
        try baseColorSource(input: .prepared(chain))
    }

    @discardableResult
    public func baseColorTexture(_ chain: Mipmap, _ options: TextureOptions) throws -> Material {
        try baseColorSource(input: .prepared(chain), options: options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ texture: Texture) throws -> Material {
        try metallicRoughnessTexture(texture: texture)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(try byteData(bytes, "Material.metallicRoughnessTexture"), usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(try byteData(bytes, "Material.metallicRoughnessTexture"), format, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(try byteData(bytes, "Material.metallicRoughnessTexture"), size, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(bytes, try sizeRecord(size, "Material.metallicRoughnessTexture"), usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try metallicRoughnessTexture(try byteData(bytes, "Material.metallicRoughnessTexture"), size, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ options: TextureOptions) throws -> Material {
        try metallicRoughnessSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ options: TextureOptions) throws -> Material {
        try metallicRoughnessTexture(Data(bytes), options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [Int], _ options: TextureOptions) throws -> Material {
        try metallicRoughnessTexture(try byteData(bytes, "Material.metallicRoughnessTexture"), options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String) throws -> Material {
        try metallicRoughnessSource(input: pathInput(path))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String, _ format: TextureFormat) throws -> Material {
        try metallicRoughnessSource(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String, _ options: TextureOptions) throws -> Material {
        try metallicRoughnessSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ chain: Mipmap) throws -> Material {
        try metallicRoughnessSource(input: .prepared(chain))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ chain: Mipmap, _ options: TextureOptions) throws -> Material {
        try metallicRoughnessSource(input: .prepared(chain), options: options)
    }

    @discardableResult
    public func normalTexture(_ texture: Texture) throws -> Material {
        try normalTexture(texture: texture)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, usage: TextureUsage? = nil) throws -> Material {
        try normalSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(try byteData(bytes, "Material.normalTexture"), usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try normalSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(try byteData(bytes, "Material.normalTexture"), format, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try normalSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(try byteData(bytes, "Material.normalTexture"), size, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(bytes, try sizeRecord(size, "Material.normalTexture"), usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try normalTexture(try byteData(bytes, "Material.normalTexture"), size, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ options: TextureOptions) throws -> Material {
        try normalSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ options: TextureOptions) throws -> Material {
        try normalTexture(Data(bytes), options)
    }

    @discardableResult
    public func normalTexture(_ bytes: [Int], _ options: TextureOptions) throws -> Material {
        try normalTexture(try byteData(bytes, "Material.normalTexture"), options)
    }

    @discardableResult
    public func normalTexture(_ path: String) throws -> Material {
        try normalSource(input: pathInput(path))
    }

    @discardableResult
    public func normalTexture(_ path: String, _ format: TextureFormat) throws -> Material {
        try normalSource(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func normalTexture(_ path: String, _ options: TextureOptions) throws -> Material {
        try normalSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func normalTexture(_ chain: Mipmap) throws -> Material {
        try normalSource(input: .prepared(chain))
    }

    @discardableResult
    public func normalTexture(_ chain: Mipmap, _ options: TextureOptions) throws -> Material {
        try normalSource(input: .prepared(chain), options: options)
    }

    @discardableResult
    public func occlusionTexture(_ texture: Texture) throws -> Material {
        try occlusionTexture(texture: texture)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, usage: TextureUsage? = nil) throws -> Material {
        try occlusionSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(try byteData(bytes, "Material.occlusionTexture"), usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try occlusionSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(try byteData(bytes, "Material.occlusionTexture"), format, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try occlusionSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(try byteData(bytes, "Material.occlusionTexture"), size, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(bytes, try sizeRecord(size, "Material.occlusionTexture"), usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try occlusionTexture(try byteData(bytes, "Material.occlusionTexture"), size, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ options: TextureOptions) throws -> Material {
        try occlusionSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ options: TextureOptions) throws -> Material {
        try occlusionTexture(Data(bytes), options)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [Int], _ options: TextureOptions) throws -> Material {
        try occlusionTexture(try byteData(bytes, "Material.occlusionTexture"), options)
    }

    @discardableResult
    public func occlusionTexture(_ path: String) throws -> Material {
        try occlusionSource(input: pathInput(path))
    }

    @discardableResult
    public func occlusionTexture(_ path: String, _ format: TextureFormat) throws -> Material {
        try occlusionSource(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func occlusionTexture(_ path: String, _ options: TextureOptions) throws -> Material {
        try occlusionSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func occlusionTexture(_ chain: Mipmap) throws -> Material {
        try occlusionSource(input: .prepared(chain))
    }

    @discardableResult
    public func occlusionTexture(_ chain: Mipmap, _ options: TextureOptions) throws -> Material {
        try occlusionSource(input: .prepared(chain), options: options)
    }

    @discardableResult
    public func emissiveTexture(_ texture: Texture) throws -> Material {
        try emissiveTexture(texture: texture)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, usage: TextureUsage? = nil) throws -> Material {
        try emissiveSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(try byteData(bytes, "Material.emissiveTexture"), usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try emissiveSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(try byteData(bytes, "Material.emissiveTexture"), format, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try emissiveSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(try byteData(bytes, "Material.emissiveTexture"), size, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(bytes, try sizeRecord(size, "Material.emissiveTexture"), usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        try emissiveTexture(try byteData(bytes, "Material.emissiveTexture"), size, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ options: TextureOptions) throws -> Material {
        try emissiveSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ options: TextureOptions) throws -> Material {
        try emissiveTexture(Data(bytes), options)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [Int], _ options: TextureOptions) throws -> Material {
        try emissiveTexture(try byteData(bytes, "Material.emissiveTexture"), options)
    }

    @discardableResult
    public func emissiveTexture(_ path: String) throws -> Material {
        try emissiveSource(input: pathInput(path))
    }

    @discardableResult
    public func emissiveTexture(_ path: String, _ format: TextureFormat) throws -> Material {
        try emissiveSource(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func emissiveTexture(_ path: String, _ options: TextureOptions) throws -> Material {
        try emissiveSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func emissiveTexture(_ chain: Mipmap) throws -> Material {
        try emissiveSource(input: .prepared(chain))
    }

    @discardableResult
    public func emissiveTexture(_ chain: Mipmap, _ options: TextureOptions) throws -> Material {
        try emissiveSource(input: .prepared(chain), options: options)
    }
}
