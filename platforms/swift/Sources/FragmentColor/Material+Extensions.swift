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

    public func metallic(_ value: Float) -> Material {
        return metallic(value: value)
    }
    public func metallic(_ value: Double) -> Material {
        return metallic(value: Float(value))
    }

    public func roughness(_ value: Float) -> Material {
        return roughness(value: value)
    }
    public func roughness(_ value: Double) -> Material {
        return roughness(value: Float(value))
    }

    public func normalScale(_ value: Float) -> Material {
        return normalScale(value: value)
    }
    public func normalScale(_ value: Double) -> Material {
        return normalScale(value: Float(value))
    }

    public func occlusionStrength(_ value: Float) -> Material {
        return occlusionStrength(value: value)
    }
    public func occlusionStrength(_ value: Double) -> Material {
        return occlusionStrength(value: Float(value))
    }

    public func alphaCutoff(_ value: Float) -> Material {
        return alphaCutoff(value: value)
    }
    public func alphaCutoff(_ value: Double) -> Material {
        return alphaCutoff(value: Float(value))
    }

    public func doubleSided(_ value: Bool) -> Material {
        return doubleSided(value: value)
    }

    // MARK: - Setters that take a Float vector or Bool.

    public func baseColor(_ color: [Float]) throws -> Material {
        return try baseColor(color: color)
    }
    public func baseColor(_ color: [Double]) throws -> Material {
        return try baseColor(color: color.map { Float($0) })
    }

    public func emissive(_ factor: [Float]) throws -> Material {
        return try emissive(factor: factor)
    }
    public func emissive(_ factor: [Double]) throws -> Material {
        return try emissive(factor: factor.map { Float($0) })
    }

    public func alphaMode(_ mode: AlphaMode) -> Material {
        return alphaMode(mode: mode)
    }

    public func uvTransform(_ offset: [Float], _ scale: [Float], _ rotation: Float) throws -> Material {
        return try uvTransform(offset: offset, scale: scale, rotation: rotation)
    }
    public func uvTransform(_ offset: [Double], _ scale: [Double], _ rotation: Double) throws -> Material {
        return try uvTransform(
            offset: offset.map { Float($0) },
            scale: scale.map { Float($0) },
            rotation: Float(rotation)
        )
    }

    // MARK: - Texture setters — drop the `texture:` label.

    public func baseColorTexture(_ texture: Texture) -> Material {
        return baseColorTexture(texture: texture)
    }

    public func metallicRoughnessTexture(_ texture: Texture) -> Material {
        return metallicRoughnessTexture(texture: texture)
    }

    public func normalTexture(_ texture: Texture) -> Material {
        return normalTexture(texture: texture)
    }

    public func occlusionTexture(_ texture: Texture) -> Material {
        return occlusionTexture(texture: texture)
    }

    public func emissiveTexture(_ texture: Texture) -> Material {
        return emissiveTexture(texture: texture)
    }
}
