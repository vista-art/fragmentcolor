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

    @discardableResult
    public func metallic(_ value: Float) -> Material {
        return metallic(value: value)
    }
    @discardableResult
    public func metallic(_ value: Double) -> Material {
        return metallic(value: Float(value))
    }

    @discardableResult
    public func roughness(_ value: Float) -> Material {
        return roughness(value: value)
    }
    @discardableResult
    public func roughness(_ value: Double) -> Material {
        return roughness(value: Float(value))
    }

    @discardableResult
    public func normalScale(_ value: Float) -> Material {
        return normalScale(value: value)
    }
    @discardableResult
    public func normalScale(_ value: Double) -> Material {
        return normalScale(value: Float(value))
    }

    @discardableResult
    public func occlusionStrength(_ value: Float) -> Material {
        return occlusionStrength(value: value)
    }
    @discardableResult
    public func occlusionStrength(_ value: Double) -> Material {
        return occlusionStrength(value: Float(value))
    }

    @discardableResult
    public func alphaCutoff(_ value: Float) -> Material {
        return alphaCutoff(value: value)
    }
    @discardableResult
    public func alphaCutoff(_ value: Double) -> Material {
        return alphaCutoff(value: Float(value))
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
            rotation: Float(rotation)
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
            rotation: Float(rotation)
        )
    }

    // MARK: - Texture setters

    // Each texture setter takes what `Renderer.createTexture` takes, spelled the same way.
    // A `Texture` binds at once, and every other input uploads when the scene loads, as in Rust.

    @discardableResult
    public func baseColorTexture(_ texture: Texture) -> Material {
        baseColorTexture(texture: texture)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, usage: TextureUsage? = nil) -> Material {
        baseColorSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) -> Material {
        baseColorTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        baseColorSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        baseColorTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) -> Material {
        baseColorSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        baseColorTexture(bytes, try sizeRecord(size, "Material.baseColorTexture"), usage: usage)
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
    public func baseColorTexture(_ bytes: Data, _ options: TextureOptions) -> Material {
        baseColorSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: [UInt8], _ options: TextureOptions) -> Material {
        baseColorTexture(Data(bytes), options)
    }

    @discardableResult
    public func baseColorTexture(_ path: String) -> Material {
        baseColorSource(input: pathInput(path))
    }

    @discardableResult
    public func baseColorTexture(_ path: String, _ options: TextureOptions) -> Material {
        baseColorSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func baseColorTexture(_ chain: Mipmap) -> Material {
        baseColorSource(input: .prepared(chain))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ texture: Texture) -> Material {
        metallicRoughnessTexture(texture: texture)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, usage: TextureUsage? = nil) -> Material {
        metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) -> Material {
        metallicRoughnessTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        metallicRoughnessTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) -> Material {
        metallicRoughnessSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        metallicRoughnessTexture(bytes, try sizeRecord(size, "Material.metallicRoughnessTexture"), usage: usage)
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
    public func metallicRoughnessTexture(_ bytes: Data, _ options: TextureOptions) -> Material {
        metallicRoughnessSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: [UInt8], _ options: TextureOptions) -> Material {
        metallicRoughnessTexture(Data(bytes), options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String) -> Material {
        metallicRoughnessSource(input: pathInput(path))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String, _ options: TextureOptions) -> Material {
        metallicRoughnessSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ chain: Mipmap) -> Material {
        metallicRoughnessSource(input: .prepared(chain))
    }

    @discardableResult
    public func normalTexture(_ texture: Texture) -> Material {
        normalTexture(texture: texture)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, usage: TextureUsage? = nil) -> Material {
        normalSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) -> Material {
        normalTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        normalSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        normalTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) -> Material {
        normalSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        normalTexture(bytes, try sizeRecord(size, "Material.normalTexture"), usage: usage)
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
    public func normalTexture(_ bytes: Data, _ options: TextureOptions) -> Material {
        normalSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func normalTexture(_ bytes: [UInt8], _ options: TextureOptions) -> Material {
        normalTexture(Data(bytes), options)
    }

    @discardableResult
    public func normalTexture(_ path: String) -> Material {
        normalSource(input: pathInput(path))
    }

    @discardableResult
    public func normalTexture(_ path: String, _ options: TextureOptions) -> Material {
        normalSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func normalTexture(_ chain: Mipmap) -> Material {
        normalSource(input: .prepared(chain))
    }

    @discardableResult
    public func occlusionTexture(_ texture: Texture) -> Material {
        occlusionTexture(texture: texture)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, usage: TextureUsage? = nil) -> Material {
        occlusionSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) -> Material {
        occlusionTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        occlusionSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        occlusionTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) -> Material {
        occlusionSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        occlusionTexture(bytes, try sizeRecord(size, "Material.occlusionTexture"), usage: usage)
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
    public func occlusionTexture(_ bytes: Data, _ options: TextureOptions) -> Material {
        occlusionSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: [UInt8], _ options: TextureOptions) -> Material {
        occlusionTexture(Data(bytes), options)
    }

    @discardableResult
    public func occlusionTexture(_ path: String) -> Material {
        occlusionSource(input: pathInput(path))
    }

    @discardableResult
    public func occlusionTexture(_ path: String, _ options: TextureOptions) -> Material {
        occlusionSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func occlusionTexture(_ chain: Mipmap) -> Material {
        occlusionSource(input: .prepared(chain))
    }

    @discardableResult
    public func emissiveTexture(_ texture: Texture) -> Material {
        emissiveTexture(texture: texture)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, usage: TextureUsage? = nil) -> Material {
        emissiveSource(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) -> Material {
        emissiveTexture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        emissiveSource(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) -> Material {
        emissiveTexture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) -> Material {
        emissiveSource(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> Material {
        emissiveTexture(bytes, try sizeRecord(size, "Material.emissiveTexture"), usage: usage)
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
    public func emissiveTexture(_ bytes: Data, _ options: TextureOptions) -> Material {
        emissiveSource(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: [UInt8], _ options: TextureOptions) -> Material {
        emissiveTexture(Data(bytes), options)
    }

    @discardableResult
    public func emissiveTexture(_ path: String) -> Material {
        emissiveSource(input: pathInput(path))
    }

    @discardableResult
    public func emissiveTexture(_ path: String, _ options: TextureOptions) -> Material {
        emissiveSource(input: pathInput(path), options: options)
    }

    @discardableResult
    public func emissiveTexture(_ chain: Mipmap) -> Material {
        emissiveSource(input: .prepared(chain))
    }
}
