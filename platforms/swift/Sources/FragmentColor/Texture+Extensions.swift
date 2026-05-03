//
//  Texture+Extensions.swift
//  FragmentColor
//
//  Convenience extensions for Texture and target types.
//  Bridges the uniffi-generated labeled API to the unlabeled call shapes
//  used in documentation examples.
//

import Foundation

extension Texture {
    // MARK: - write convenience

    /// Upload raw pixel data into the whole texture (unlabeled overload).
    public func write(_ bytes: Data) throws {
        try write(bytes: bytes)
    }

    /// Upload a `[UInt8]` byte array into the whole texture.
    public func write(_ bytes: [UInt8]) throws {
        try write(bytes: Data(bytes))
    }

    /// Upload a `[Int]` byte array into the whole texture (values clamped to UInt8).
    public func write(_ bytes: [Int]) throws {
        try write(Data(bytes.map { UInt8(clamping: $0) }))
    }

    // MARK: - writeRegion convenience

    /// Upload pixel data into a sub-region using a `[x, y, width, height]` array.
    public func writeRegion(_ bytes: Data, _ rect: [Int]) throws {
        guard rect.count == 4 else { return }
        let region = TextureRegionMobile(
            originX: UInt32(rect[0]), originY: UInt32(rect[1]), originZ: 0,
            sizeWidth: UInt32(rect[2]), sizeHeight: UInt32(rect[3]), sizeDepth: 1,
            bytesPerRow: nil, rowsPerImage: nil
        )
        try writeRegion(bytes: bytes, region: region)
    }

    public func writeRegion(_ bytes: [UInt8], _ rect: [Int]) throws {
        try writeRegion(Data(bytes), rect)
    }

    public func writeRegion(_ bytes: [Int], _ rect: [Int]) throws {
        try writeRegion(bytes.map { UInt8(clamping: $0) }, rect)
    }

    // MARK: - setSamplerOptions convenience

    public func setSamplerOptions(_ opts: SamplerOptions) {
        setSamplerOptions(opts: opts)
    }

    // MARK: - id convenience

    /// Return the raw UInt64 numeric id (convenience over `.id().id`).
    public var rawId: UInt64 {
        return id().id
    }
}

extension MobileTextureTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    public func resize(_ size: [Int]) {
        guard size.count >= 2 else { return }
        resize(width: UInt32(size[0]), height: UInt32(size[1]))
    }

    public func resize(_ size: [UInt32]) {
        guard size.count >= 2 else { return }
        resize(width: size[0], height: size[1])
    }
}

extension MobileWindowTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    public func resize(_ size: [Int]) {
        guard size.count >= 2 else { return }
        resize(width: UInt32(size[0]), height: UInt32(size[1]))
    }

    public func resize(_ size: [UInt32]) {
        guard size.count >= 2 else { return }
        resize(width: size[0], height: size[1])
    }
}
