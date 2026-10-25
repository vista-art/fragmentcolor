//
//  Texture+Extensions.swift
//  FragmentColor
//
//  Convenience extensions for Texture and target types.
//  Bridges the uniffi-generated labeled API to the unlabeled call shapes
//  used in documentation examples.
//

import Foundation

@available(*, deprecated, renamed: "TextureTarget")
public typealias MobileTextureTarget = TextureTarget

@available(*, deprecated, renamed: "WindowTarget")
public typealias MobileWindowTarget = WindowTarget

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

    /// Upload one `Int` per byte into the whole texture. Throws for a value outside 0...255.
    public func write(_ bytes: [Int]) throws {
        try write(try byteData(bytes, "write"))
    }

    // MARK: - writeRegion convenience

    /// Upload pixel data into a region given as `[w, h]`, `[x, y, w, h]` or `[x, y, z, w, h, d]`.
    /// Throws for any other length.
    public func writeRegion(_ bytes: Data, _ rect: [UInt32]) throws {
        guard let region = textureRegion(rect) else {
            throw FragmentColorError.Render("writeRegion: expected 2, 4 or 6 numbers, got \(rect.count)")
        }
        try writeRegion(bytes: bytes, region: region)
    }

    public func writeRegion(_ bytes: [UInt8], _ rect: [UInt32]) throws {
        try writeRegion(Data(bytes), rect)
    }

    public func writeRegion(_ bytes: [Int], _ rect: [UInt32]) throws {
        try writeRegion(try byteData(bytes, "writeRegion"), rect)
    }

    // `TextureRegion` overloads, for a region built with
    // `TextureRegion.from(_:).withStride(_:).withRows(_:)`.

    public func writeRegion(_ bytes: Data, _ region: TextureRegion) throws {
        try writeRegion(bytes: bytes, region: region)
    }

    public func writeRegion(_ bytes: [UInt8], _ region: TextureRegion) throws {
        try writeRegion(Data(bytes), region)
    }

    public func writeRegion(_ bytes: [Int], _ region: TextureRegion) throws {
        try writeRegion(try byteData(bytes, "writeRegion"), region)
    }

    /// Upload pixel data into the pixels a `ScreenRegion` covers.
    public func writeRegion(_ bytes: Data, _ region: ScreenRegion) throws {
        try writeRegion(bytes, TextureRegion.from(region))
    }

    public func writeRegion(_ bytes: [UInt8], _ region: ScreenRegion) throws {
        try writeRegion(Data(bytes), region)
    }

    public func writeRegion(_ bytes: [Int], _ region: ScreenRegion) throws {
        try writeRegion(try byteData(bytes, "writeRegion"), region)
    }

    // MARK: - setSamplerOptions convenience

    public func setSamplerOptions(_ opts: SamplerOptions) {
        setSamplerOptions(opts: opts)
    }

    // MARK: - resize convenience

    /// Resize the texture from a `[width, height]` array. Throws for any other length.
    public func resize(_ size: [UInt32]) throws {
        try resize(size: try sizeRecord(size, "resize"))
    }

    /// Resize the texture to `size`, such as a target's `size()`.
    public func resize(_ size: Size) throws {
        try resize(size: size)
    }

    // MARK: - id convenience

    /// Return the raw UInt64 numeric id (convenience over `.id().id`).
    public var rawId: UInt64 {
        return id().id
    }
}

extension TextureTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    /// Throws for any other length, or when the target can't take the size.
    public func resize(_ size: [UInt32]) throws {
        try resize(try sizeRecord(size, "resize"))
    }

    /// Resize the target to `size`, such as another target's `size()`.
    /// Throws when the target can't take it.
    public func resize(_ size: Size) throws {
        try resize(size: size)
    }
}

extension WindowTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    /// Throws for any other length, or when the target can't take the size.
    public func resize(_ size: [UInt32]) throws {
        try resize(try sizeRecord(size, "resize"))
    }

    /// Resize the target to `size`, such as another target's `size()`.
    /// Throws when the target can't take it.
    public func resize(_ size: Size) throws {
        try resize(size: size)
    }
}

extension Mipmap {
    /// Decode encoded image bytes (PNG, JPEG and the other formats FragmentColor reads)
    /// into a chain of `format`.
    public static func build(_ bytes: Data, _ format: TextureFormat) throws -> Mipmap {
        try Mipmap.build(bytes: bytes, format: format, size: nil)
    }

    public static func build(_ bytes: [UInt8], _ format: TextureFormat) throws -> Mipmap {
        try Mipmap.build(Data(bytes), format)
    }

    /// Build a chain from raw pixels of `format` and their `Size`.
    public static func build(_ bytes: Data, _ format: TextureFormat, _ size: Size) throws -> Mipmap {
        try Mipmap.build(bytes: bytes, format: format, size: size)
    }

    /// Build a chain from raw pixels of `format` and their `[width, height]`.
    public static func build(_ bytes: Data, _ format: TextureFormat, _ size: [UInt32]) throws -> Mipmap {
        try Mipmap.build(bytes, format, try sizeRecord(size, "Mipmap.build"))
    }

    public static func build(_ bytes: [UInt8], _ format: TextureFormat, _ size: [UInt32]) throws -> Mipmap {
        try Mipmap.build(Data(bytes), format, size)
    }

    /// The bytes of one mip level (unlabeled overload). Throws for an index out of range.
    public func level(_ index: UInt32) throws -> Data {
        try level(index: index)
    }
}

extension TextureUsage {
    /// The texture can be the source of a copy, such as `Renderer.readTexture`.
    public static let COPY_SRC = TextureUsage(bits: 1 << 0)
    /// The texture can be the destination of a copy, such as `Texture.write`.
    public static let COPY_DST = TextureUsage(bits: 1 << 1)
    /// A shader can sample the texture or load from it.
    public static let TEXTURE_BINDING = TextureUsage(bits: 1 << 2)
    /// A shader can write the texture as a storage texture.
    public static let STORAGE_BINDING = TextureUsage(bits: 1 << 3)
    /// A pass can render into the texture.
    public static let RENDER_ATTACHMENT = TextureUsage(bits: 1 << 4)

    /// Both sets of flags, as Rust's `|` combines them.
    public static func | (lhs: TextureUsage, rhs: TextureUsage) -> TextureUsage {
        TextureUsage(bits: lhs.bits | rhs.bits)
    }
}
