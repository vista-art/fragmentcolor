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
    public func baseColor(_ color: [Double]) throws -> Material {
        return try baseColor(color: color.map { Float($0) })
    }

    @discardableResult
    public func emissive(_ factor: [Float]) throws -> Material {
        return try emissive(factor: factor)
    }
    @discardableResult
    public func emissive(_ factor: [Double]) throws -> Material {
        return try emissive(factor: factor.map { Float($0) })
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
    public func uvTransform(_ offset: [Double], _ scale: [Double], _ rotation: Double) throws -> Material {
        return try uvTransform(
            offset: offset.map { Float($0) },
            scale: scale.map { Float($0) },
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
    public func textureTransform(
        _ slot: TextureSlot, _ offset: [Double], _ scale: [Double], _ rotation: Double
    ) throws -> Material {
        return try textureTransform(
            slot: slot,
            offset: offset.map { Float($0) },
            scale: scale.map { Float($0) },
            rotation: Float(rotation)
        )
    }

    // MARK: - Texture setters — drop the `texture:` label.
    //
    // Each setter takes what `Renderer.createTexture` takes: a `Texture`, or
    // encoded bytes, a file path or `http(s)` URL, or a `Mipmap`. Inputs other
    // than a `Texture` upload when the scene loads, as in Rust.

    @discardableResult
    public func baseColorTexture(_ texture: Texture) -> Material {
        return baseColorTexture(texture: texture)
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data) -> Material {
        return baseColorSource(input: .bytes(bytes))
    }

    @discardableResult
    public func baseColorTexture(_ bytes: Data, _ format: TextureFormat) -> Material {
        return baseColorSource(input: .bytes(bytes), options: formatOptions(format))
    }

    @discardableResult
    public func baseColorTexture(_ path: String) -> Material {
        return baseColorSource(input: textureSource(path))
    }

    @discardableResult
    public func baseColorTexture(_ chain: Mipmap) -> Material {
        return baseColorSource(input: .prepared(chain))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ texture: Texture) -> Material {
        return metallicRoughnessTexture(texture: texture)
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data) -> Material {
        return metallicRoughnessSource(input: .bytes(bytes))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ bytes: Data, _ format: TextureFormat) -> Material {
        return metallicRoughnessSource(input: .bytes(bytes), options: formatOptions(format))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ path: String) -> Material {
        return metallicRoughnessSource(input: textureSource(path))
    }

    @discardableResult
    public func metallicRoughnessTexture(_ chain: Mipmap) -> Material {
        return metallicRoughnessSource(input: .prepared(chain))
    }

    @discardableResult
    public func normalTexture(_ texture: Texture) -> Material {
        return normalTexture(texture: texture)
    }

    @discardableResult
    public func normalTexture(_ bytes: Data) -> Material {
        return normalSource(input: .bytes(bytes))
    }

    @discardableResult
    public func normalTexture(_ bytes: Data, _ format: TextureFormat) -> Material {
        return normalSource(input: .bytes(bytes), options: formatOptions(format))
    }

    @discardableResult
    public func normalTexture(_ path: String) -> Material {
        return normalSource(input: textureSource(path))
    }

    @discardableResult
    public func normalTexture(_ chain: Mipmap) -> Material {
        return normalSource(input: .prepared(chain))
    }

    @discardableResult
    public func occlusionTexture(_ texture: Texture) -> Material {
        return occlusionTexture(texture: texture)
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data) -> Material {
        return occlusionSource(input: .bytes(bytes))
    }

    @discardableResult
    public func occlusionTexture(_ bytes: Data, _ format: TextureFormat) -> Material {
        return occlusionSource(input: .bytes(bytes), options: formatOptions(format))
    }

    @discardableResult
    public func occlusionTexture(_ path: String) -> Material {
        return occlusionSource(input: textureSource(path))
    }

    @discardableResult
    public func occlusionTexture(_ chain: Mipmap) -> Material {
        return occlusionSource(input: .prepared(chain))
    }

    @discardableResult
    public func emissiveTexture(_ texture: Texture) -> Material {
        return emissiveTexture(texture: texture)
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data) -> Material {
        return emissiveSource(input: .bytes(bytes))
    }

    @discardableResult
    public func emissiveTexture(_ bytes: Data, _ format: TextureFormat) -> Material {
        return emissiveSource(input: .bytes(bytes), options: formatOptions(format))
    }

    @discardableResult
    public func emissiveTexture(_ path: String) -> Material {
        return emissiveSource(input: textureSource(path))
    }

    @discardableResult
    public func emissiveTexture(_ chain: Mipmap) -> Material {
        return emissiveSource(input: .prepared(chain))
    }
}

/// A URL input for `http(s)`, otherwise a file path.
private func textureSource(_ path: String) -> TextureInputMobile {
    path.hasPrefix("http://") || path.hasPrefix("https://") ? .url(path) : .path(path)
}

/// Rust's default `TextureOptions` with `format`.
private func formatOptions(_ format: TextureFormat) -> TextureOptions {
    TextureOptions(
        size: nil,
        format: format,
        sampler: SamplerOptions(repeatX: false, repeatY: false, smooth: true, compare: nil),
        mipmaps: true,
        usage: nil
    )
}
