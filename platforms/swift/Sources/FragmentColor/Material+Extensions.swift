//
//  Material+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated material API:
//  PbrMaterial and the MaterialMap its map calls return.
//  Mirrors the spelling used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

extension PbrMaterial {
    // MARK: - Setters that take a single scalar Float.
    //
    // `metallic(value:)`, `roughness(value:)`, `normalScale(value:)`,
    // `occlusionStrength(value:)`, `alphaCutoff(value:)`, `doubleSided(value:)`
    // — accept positional `Float` / `Double` / `Bool` so example code can
    // write `mat.metallic(0.5)` instead of `mat.metallic(value: 0.5)`.
    // A number setter throws for NaN, an infinity or a `Double` past the `Float` range.

    @discardableResult
    public func metallic(_ value: Float) throws -> PbrMaterial {
        return try metallic(value: value)
    }
    @discardableResult
    public func metallic(_ value: Double) throws -> PbrMaterial {
        return try metallic(value: readFinite(value, "PbrMaterial.metallic: value"))
    }

    @discardableResult
    public func roughness(_ value: Float) throws -> PbrMaterial {
        return try roughness(value: value)
    }
    @discardableResult
    public func roughness(_ value: Double) throws -> PbrMaterial {
        return try roughness(value: readFinite(value, "PbrMaterial.roughness: value"))
    }

    @discardableResult
    public func normalScale(_ value: Float) throws -> PbrMaterial {
        return try normalScale(value: value)
    }
    @discardableResult
    public func normalScale(_ value: Double) throws -> PbrMaterial {
        return try normalScale(value: readFinite(value, "PbrMaterial.normalScale: value"))
    }

    @discardableResult
    public func occlusionStrength(_ value: Float) throws -> PbrMaterial {
        return try occlusionStrength(value: value)
    }
    @discardableResult
    public func occlusionStrength(_ value: Double) throws -> PbrMaterial {
        return try occlusionStrength(value: readFinite(value, "PbrMaterial.occlusionStrength: value"))
    }

    @discardableResult
    public func alphaCutoff(_ value: Float) throws -> PbrMaterial {
        return try alphaCutoff(value: value)
    }
    @discardableResult
    public func alphaCutoff(_ value: Double) throws -> PbrMaterial {
        return try alphaCutoff(value: readFinite(value, "PbrMaterial.alphaCutoff: value"))
    }

    @discardableResult
    public func doubleSided(_ value: Bool) -> PbrMaterial {
        return doubleSided(value: value)
    }

    // MARK: - Setters that take a Float vector or a mode.

    @discardableResult
    public func baseColor(_ color: [Float]) throws -> PbrMaterial {
        return try baseColor(color: color)
    }
    @discardableResult
    @_disfavoredOverload
    public func baseColor(_ color: [Double]) throws -> PbrMaterial {
        return try baseColor(color: readFloats(color, "PbrMaterial.baseColor"))
    }

    @discardableResult
    public func emissive(_ factor: [Float]) throws -> PbrMaterial {
        return try emissive(factor: factor)
    }
    @discardableResult
    @_disfavoredOverload
    public func emissive(_ factor: [Double]) throws -> PbrMaterial {
        return try emissive(factor: readFloats(factor, "PbrMaterial.emissive"))
    }

    @discardableResult
    public func alphaMode(_ mode: AlphaMode) -> PbrMaterial {
        return alphaMode(mode: mode)
    }

    @discardableResult
    public func uvTransform(_ offset: [Float], _ scale: [Float], _ rotation: Float) throws -> PbrMaterial {
        return try uvTransform(offset: offset, scale: scale, rotation: rotation)
    }
    @discardableResult
    @_disfavoredOverload
    public func uvTransform(_ offset: [Double], _ scale: [Double], _ rotation: Double) throws -> PbrMaterial {
        return try uvTransform(
            offset: readFloats(offset, "PbrMaterial.uvTransform: offset"),
            scale: readFloats(scale, "PbrMaterial.uvTransform: scale"),
            rotation: readFinite(rotation, "PbrMaterial.uvTransform: rotation")
        )
    }
}

extension MaterialMap {
    // MARK: - UV set and transform

    @discardableResult
    public func uv(_ set: UInt32) throws -> MaterialMap {
        return try uv(set: set)
    }

    @discardableResult
    public func transform(_ offset: [Float], _ scale: [Float], _ rotation: Float) throws -> MaterialMap {
        return try transform(offset: offset, scale: scale, rotation: rotation)
    }
    @discardableResult
    @_disfavoredOverload
    public func transform(_ offset: [Double], _ scale: [Double], _ rotation: Double) throws -> MaterialMap {
        return try transform(
            offset: readFloats(offset, "MaterialMap.transform: offset"),
            scale: readFloats(scale, "MaterialMap.transform: scale"),
            rotation: readFinite(rotation, "MaterialMap.transform: rotation")
        )
    }

    // MARK: - Texture

    // `texture` takes what `Renderer.createTexture` takes, spelled the same way.
    // A `Texture` binds at once, and every other input uploads when the scene loads, as in Rust.

    @discardableResult
    public func texture(_ texture: Texture) throws -> MaterialMap {
        try self.texture(texture: texture)
    }

    @discardableResult
    public func texture(_ bytes: Data, usage: TextureUsage? = nil) throws -> MaterialMap {
        try source(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    @discardableResult
    public func texture(_ bytes: [UInt8], usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(Data(bytes), usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: [Int], usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(try byteData(bytes, "MaterialMap.texture"), usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) throws -> MaterialMap {
        try source(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    @discardableResult
    public func texture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(Data(bytes), format, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: [Int], _ format: TextureFormat, usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(try byteData(bytes, "MaterialMap.texture"), format, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) throws -> MaterialMap {
        try source(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    @discardableResult
    public func texture(_ bytes: [UInt8], _ size: Size, usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: [Int], _ size: Size, usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(try byteData(bytes, "MaterialMap.texture"), size, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(bytes, try sizeRecord(size, "MaterialMap.texture"), usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(Data(bytes), size, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) throws -> MaterialMap {
        try texture(try byteData(bytes, "MaterialMap.texture"), size, usage: usage)
    }

    @discardableResult
    public func texture(_ bytes: Data, _ options: TextureOptions) throws -> MaterialMap {
        try source(input: .bytes(bytes), options: options)
    }

    @discardableResult
    public func texture(_ bytes: [UInt8], _ options: TextureOptions) throws -> MaterialMap {
        try texture(Data(bytes), options)
    }

    @discardableResult
    public func texture(_ bytes: [Int], _ options: TextureOptions) throws -> MaterialMap {
        try texture(try byteData(bytes, "MaterialMap.texture"), options)
    }

    @discardableResult
    public func texture(_ path: String) throws -> MaterialMap {
        try source(input: pathInput(path))
    }

    @discardableResult
    public func texture(_ path: String, _ format: TextureFormat) throws -> MaterialMap {
        try source(input: pathInput(path), options: textureOptions(format: format))
    }

    @discardableResult
    public func texture(_ path: String, _ options: TextureOptions) throws -> MaterialMap {
        try source(input: pathInput(path), options: options)
    }

    @discardableResult
    public func texture(_ chain: Mipmap) throws -> MaterialMap {
        try source(input: .prepared(chain))
    }

    @discardableResult
    public func texture(_ chain: Mipmap, _ options: TextureOptions) throws -> MaterialMap {
        try source(input: .prepared(chain), options: options)
    }
}
