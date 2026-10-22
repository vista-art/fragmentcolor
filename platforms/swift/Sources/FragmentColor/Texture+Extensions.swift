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
    /// Throws for any other length or a negative number.
    public func writeRegion(_ bytes: Data, _ rect: [Int]) throws {
        guard let region = textureRegion(rect) else {
            throw FragmentColorError.Render("writeRegion: expected 2, 4 or 6 non-negative numbers, got \(rect)")
        }
        try writeRegion(bytes: bytes, region: region)
    }

    public func writeRegion(_ bytes: [UInt8], _ rect: [Int]) throws {
        try writeRegion(Data(bytes), rect)
    }

    public func writeRegion(_ bytes: [Int], _ rect: [Int]) throws {
        try writeRegion(try byteData(bytes, "writeRegion"), rect)
    }

    // Pre-built `TextureRegion` overloads, for the
    // `TextureRegion.from(_:).withStride(_:).withRows(_:)` builder chain
    // in `write_region.md`. Without them the Swift overload resolver picks
    // the `[Int]` overload above and rejects a `TextureRegion` argument.

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

    // MARK: - id convenience

    /// Return the raw UInt64 numeric id (convenience over `.id().id`).
    public var rawId: UInt64 {
        return id().id
    }
}

extension TextureTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    /// A length other than 2 or 3, or a negative number, stops the program.
    public func resize(_ size: [Int]) {
        resize(targetSize(size))
    }

    public func resize(_ size: [UInt32]) {
        resize(targetSize(size))
    }

    /// Resize the target to `size`, such as another target's `size()`.
    public func resize(_ size: Size) {
        resize(width: size.width, height: size.height)
    }
}

extension WindowTarget {
    // MARK: - resize convenience

    /// Resize the target from a `[width, height]` array.
    /// A length other than 2 or 3, or a negative number, stops the program.
    public func resize(_ size: [Int]) {
        resize(targetSize(size))
    }

    public func resize(_ size: [UInt32]) {
        resize(targetSize(size))
    }

    /// Resize the target to `size`, such as another target's `size()`.
    public func resize(_ size: Size) {
        resize(width: size.width, height: size.height)
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
    public static func build(_ bytes: Data, _ format: TextureFormat, _ size: [Int]) throws -> Mipmap {
        try Mipmap.build(bytes, format, try sizeRecord(size, "Mipmap.build"))
    }

    public static func build(_ bytes: [UInt8], _ format: TextureFormat, _ size: [Int]) throws -> Mipmap {
        try Mipmap.build(Data(bytes), format, size)
    }

    /// The bytes of one mip level (unlabeled overload). Throws for an index out of range.
    public func level(_ index: Int) throws -> Data {
        guard let index = UInt32(exactly: index) else {
            throw FragmentColorError.Render("level: \(index) is not a level index")
        }
        return try level(index: index)
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

/// The `Size` a resize array names. A bad array stops the program,
/// since the resize methods return nothing to throw through.
private func targetSize<T: BinaryInteger>(_ size: [T]) -> Size {
    do {
        return try sizeRecord(size, "resize")
    } catch {
        preconditionFailure("\(error)")
    }
}
