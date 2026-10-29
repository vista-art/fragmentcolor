//
//  Renderer+Extensions.swift
//  FragmentColor
//
//  Idiomatic Swift wrappers on top of the uniffi-generated API.
//  Matches the struct / method names used by the JavaScript and Python bindings.
//

import Foundation
import QuartzCore

extension Renderer {
    // MARK: - createTarget

    /// Build a `WindowTarget` from a `CAMetalLayer`. The uniffi-generated
    /// `createTarget(metalLayerPtr:)` only accepts a raw pointer because
    /// uniffi cannot marshal `CAMetalLayer` directly; this overload does
    /// the pointer extraction so call sites read naturally.
    public func createTarget(layer: CAMetalLayer) throws -> WindowTarget {
        let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
        return try self.createTarget(metalLayerPtr: ptr)
    }

    /// Build a `WindowTarget` from a `CAMetalLayer`, unlabeled like every other platform's call.
    public func createTarget(_ layer: CAMetalLayer) throws -> WindowTarget {
        try createTarget(layer: layer)
    }

    // MARK: - createTextureTarget

    /// Create a headless texture target from a `[width, height]` array.
    public func createTextureTarget(_ size: [UInt32]) async throws -> TextureTarget {
        try await createTextureTarget(try sizeRecord(size, "createTextureTarget"))
    }

    /// Create a headless texture target from a `Size`.
    public func createTextureTarget(_ size: Size) async throws -> TextureTarget {
        try await createTextureTarget(size: size)
    }

    // MARK: - createDepthTexture

    /// Create a depth texture from a `[width, height]` array.
    public func createDepthTexture(_ size: [UInt32]) async throws -> Texture {
        try await createDepthTexture(try sizeRecord(size, "createDepthTexture"))
    }

    /// Create a depth texture from a `Size`.
    public func createDepthTexture(_ size: Size) async throws -> Texture {
        try await createDepthTexture(size: size)
    }

    // MARK: - createStorageTexture

    /// Create a storage texture of `size` and `format`, optionally seeded with `data`.
    /// `usage` replaces the default `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC | COPY_DST`.
    public func createStorageTexture(
        _ size: Size,
        _ format: TextureFormat,
        _ data: Data? = nil,
        _ usage: TextureUsage? = nil
    ) async throws -> Texture {
        try await createStorageTexture(size: size, format: format, data: data, usageBits: usage?.bits)
    }

    /// Create a storage texture from a `[width, height]` or `[width, height, depth]` array.
    public func createStorageTexture(
        _ size: [UInt32],
        _ format: TextureFormat,
        _ data: Data? = nil,
        _ usage: TextureUsage? = nil
    ) async throws -> Texture {
        let size = try sizeRecord(size, "createStorageTexture")
        return try await createStorageTexture(size, format, data, usage)
    }

    /// Create a storage texture seeded with `[UInt8]` bytes.
    public func createStorageTexture(
        _ size: [UInt32],
        _ format: TextureFormat,
        _ data: [UInt8],
        _ usage: TextureUsage? = nil
    ) async throws -> Texture {
        try await createStorageTexture(size, format, Data(data), usage)
    }

    /// Create a storage texture seeded with one `Int` per byte.
    public func createStorageTexture(
        _ size: [UInt32],
        _ format: TextureFormat,
        _ data: [Int],
        _ usage: TextureUsage? = nil
    ) async throws -> Texture {
        try await createStorageTexture(size, format, try byteData(data, "createStorageTexture"), usage)
    }

    @available(*, deprecated, message: "Pass the size and format as separate arguments: createStorageTexture(size, format)")
    public func createStorageTexture(_ sizeAndFormat: ([UInt32], TextureFormat)) async throws -> Texture {
        try await createStorageTexture(sizeAndFormat.0, sizeAndFormat.1)
    }

    @available(*, deprecated, message: "Pass the size, format and bytes as separate arguments: createStorageTexture(size, format, bytes)")
    public func createStorageTexture(_ sizeFormatData: ([UInt32], TextureFormat, [Int])) async throws -> Texture {
        try await createStorageTexture(sizeFormatData.0, sizeFormatData.1, sizeFormatData.2)
    }

    @available(*, deprecated, message: "Pass the size, format and bytes as separate arguments: createStorageTexture(size, format, bytes)")
    public func createStorageTexture(_ sizeFormatData: ([UInt32], TextureFormat, [UInt8])) async throws -> Texture {
        try await createStorageTexture(sizeFormatData.0, sizeFormatData.1, sizeFormatData.2)
    }

    // MARK: - createTexture

    /// Create a texture from encoded image bytes (PNG, JPEG and the other formats FragmentColor decodes).
    /// `usage` adds to the `TEXTURE_BINDING | COPY_DST` every texture gets.
    public func createTexture(_ bytes: Data, usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(input: .bytes(bytes), options: textureOptions(usage: usage))
    }

    /// Create a texture from encoded image bytes given as `[UInt8]`.
    public func createTexture(_ bytes: [UInt8], usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(Data(bytes), usage: usage)
    }

    /// Create a texture from encoded image bytes, decoded into `format`.
    public func createTexture(_ bytes: Data, _ format: TextureFormat, usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(input: .bytes(bytes), options: textureOptions(format: format, usage: usage))
    }

    /// Create a texture from encoded image bytes given as `[UInt8]`, decoded into `format`.
    public func createTexture(_ bytes: [UInt8], _ format: TextureFormat, usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(Data(bytes), format, usage: usage)
    }

    /// Create a texture from raw RGBA pixels and their `Size`, so the bytes are never decoded.
    public func createTexture(_ bytes: Data, _ size: Size, usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(input: .bytes(bytes), options: textureOptions(size: size, usage: usage))
    }

    /// Create a texture from raw RGBA pixels and their `[width, height]`.
    public func createTexture(_ bytes: Data, _ size: [UInt32], usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(bytes, try sizeRecord(size, "createTexture"), usage: usage)
    }

    /// Create a texture from raw RGBA pixels given as `[UInt8]`, and their `[width, height]`.
    public func createTexture(_ bytes: [UInt8], _ size: [UInt32], usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(Data(bytes), size, usage: usage)
    }

    /// Create a texture from raw RGBA pixels, one `Int` per byte, and their `[width, height]`.
    public func createTexture(_ bytes: [Int], _ size: [UInt32], usage: TextureUsage? = nil) async throws -> Texture {
        try await createTexture(try byteData(bytes, "createTexture"), size, usage: usage)
    }

    /// Create a texture from bytes with every option spelled out.
    public func createTexture(_ bytes: Data, _ options: TextureOptions) async throws -> Texture {
        try await createTexture(input: .bytes(bytes), options: options)
    }

    /// Create a texture from `[UInt8]` bytes with every option spelled out.
    public func createTexture(_ bytes: [UInt8], _ options: TextureOptions) async throws -> Texture {
        try await createTexture(Data(bytes), options)
    }

    /// Create a texture from an image file path or an `http(s)` URL.
    public func createTexture(_ path: String) async throws -> Texture {
        try await createTexture(input: pathInput(path), options: nil)
    }

    /// Create a texture from an image file path or an `http(s)` URL, with every option spelled out.
    public func createTexture(_ path: String, _ options: TextureOptions) async throws -> Texture {
        try await createTexture(input: pathInput(path), options: options)
    }

    /// Create a texture from a `Mipmap`.
    public func createTexture(_ chain: Mipmap) async throws -> Texture {
        try await createTexture(input: .prepared(chain), options: nil)
    }

    /// Default-options overload: drop the trailing `options: nil` from
    /// idiomatic call sites. Mirrors the JS / Python bindings where
    /// `create_texture(input)` is fine without an explicit options arg.
    public func createTexture(input: TextureInputMobile) async throws -> Texture {
        try await createTexture(input: input, options: nil)
    }

    @available(*, deprecated, message: "Pass the bytes and size as separate arguments: createTexture(bytes, size)")
    public func createTexture(_ bytesAndSize: ([UInt8], [UInt32])) async throws -> Texture {
        try await createTexture(bytesAndSize.0, bytesAndSize.1)
    }

    @available(*, deprecated, message: "Pass the bytes and size as separate arguments: createTexture(bytes, size)")
    public func createTexture(_ bytesAndSize: ([Int], [UInt32])) async throws -> Texture {
        try await createTexture(bytesAndSize.0, bytesAndSize.1)
    }

    // MARK: - unregisterTexture convenience

    /// Unregister a texture by its `TextureId` record.
    public func unregisterTexture(_ id: TextureId) throws {
        try unregisterTexture(textureId: id.id)
    }

    // MARK: - readStorage / readTexture convenience

    /// Read back a shader's storage binding by `(shader, binding)` pair.
    /// Mirrors the JS / Python call shape so doc examples read identically.
    public func readStorage(_ shader: Shader, _ binding: String) async throws -> Data {
        return try await readStorage(shader: shader, binding: binding)
    }

    /// Read back a registered texture by its `TextureId` (raw u64 underneath).
    /// Mirrors the JS / Python call shape so doc examples read identically.
    public func readTexture(_ id: TextureId) async throws -> Data {
        return try await readTexture(textureId: id.id)
    }

    // MARK: - render (unlabeled overloads)

    /// Single overloaded `render(...)` family that matches the spelling used
    /// by the JavaScript and Python bindings. The uniffi layer exports one
    /// concrete `render(renderable:target:)` method that takes
    /// `RenderableHandle` + `TargetHandle` enums — these extensions wrap the
    /// native types into the matching variants invisibly so callers just
    /// write `try renderer.render(shader, target)` (or `pass`, `mesh`,
    /// `passList`).
    public func render(_ shader: Shader, _ target: WindowTarget) throws {
        try self.render(renderable: .shader(shader), target: .window(target))
    }

    public func render(_ shader: Shader, _ target: TextureTarget) throws {
        try self.render(renderable: .shader(shader), target: .texture(target))
    }

    public func render(_ pass: Pass, _ target: WindowTarget) throws {
        try self.render(renderable: .pass(pass), target: .window(target))
    }

    public func render(_ pass: Pass, _ target: TextureTarget) throws {
        try self.render(renderable: .pass(pass), target: .texture(target))
    }

    public func render(_ mesh: Mesh, _ target: WindowTarget) throws {
        try self.render(renderable: .mesh(mesh), target: .window(target))
    }

    public func render(_ mesh: Mesh, _ target: TextureTarget) throws {
        try self.render(renderable: .mesh(mesh), target: .texture(target))
    }

    public func render(_ passes: [Pass], _ target: WindowTarget) throws {
        try self.render(renderable: .passes(passes), target: .window(target))
    }

    public func render(_ passes: [Pass], _ target: TextureTarget) throws {
        try self.render(renderable: .passes(passes), target: .texture(target))
    }

    public func render(_ scene: Scene, _ target: WindowTarget) throws {
        try self.render(renderable: .scene(scene), target: .window(target))
    }

    public func render(_ scene: Scene, _ target: TextureTarget) throws {
        try self.render(renderable: .scene(scene), target: .texture(target))
    }

    // MARK: - load (unlabeled overloads)
    //
    // Mirrors the `render(...)` family above. The uniffi-generated
    // `load(renderable:)` takes a `RenderableHandle`; these overloads wrap
    // each concrete type into its variant so doc examples can write
    // `try await renderer.load(scene)` (or `pass`, `mesh`, `shader`,
    // `[pass1, pass2]`) the same way every other binding does.

    public func load(_ shader: Shader) async throws {
        try await self.load(renderable: .shader(shader))
    }

    public func load(_ pass: Pass) async throws {
        try await self.load(renderable: .pass(pass))
    }

    public func load(_ mesh: Mesh) async throws {
        try await self.load(renderable: .mesh(mesh))
    }

    public func load(_ passes: [Pass]) async throws {
        try await self.load(renderable: .passes(passes))
    }

    public func load(_ scene: Scene) async throws {
        try await self.load(renderable: .scene(scene))
    }
}

/// A `Size` from `[width, height]` or `[width, height, depth]`.
/// Throws for any other length.
func sizeRecord(_ size: [UInt32], _ method: String) throws -> Size {
    guard size.count == 2 || size.count == 3 else {
        throw FragmentColorError.Render("\(method): a size holds 2 or 3 numbers, got \(size.count)")
    }
    return Size(width: size[0], height: size[1], depth: size.count == 3 ? size[2] : nil)
}

/// Bytes from one `Int` per byte. Throws for a value outside 0...255.
func byteData(_ bytes: [Int], _ method: String) throws -> Data {
    Data(try bytes.map { value -> UInt8 in
        guard let byte = UInt8(exactly: value) else {
            throw FragmentColorError.Render("\(method): \(value) is not a byte")
        }
        return byte
    })
}

/// Each number of a color or vector as a `Float`, the rule every binding shares.
/// Throws for NaN, an infinity, or a finite number past the `Float` range.
func readFloats(_ values: [Double], _ label: String) throws -> [Float] {
    try values.enumerated().map { index, value in
        let float = Float(value)
        guard float.isFinite else {
            throw FragmentColorError.Render(
                "\(label): item \(index) must be a finite number that fits a 32-bit float, got \(showNumber(value))"
            )
        }
        return float
    }
}

/// A vertex position of 1 to 4 numbers as Floats, each checked as `readFloats` checks it.
func readPosition(_ values: [Double], _ label: String) throws -> [Float] {
    let floats = try readFloats(values, label)
    guard (1...4).contains(floats.count) else {
        throw FragmentColorError.Render("\(label) takes 1 to 4 numbers, got \(floats.count)")
    }
    return floats
}

/// Exactly `count` numbers of a float vector as Floats, each checked as `readFloats` checks it.
func readVector(_ values: [Double], _ count: Int, _ label: String) throws -> [Float] {
    let floats = try readFloats(values, label)
    guard floats.count == count else {
        throw FragmentColorError.Render("\(label) takes \(count) numbers, got \(floats.count)")
    }
    return floats
}

/// Exactly `count` Floats of a float vector, checked as the Double one is.
func readVector(_ values: [Float], _ count: Int, _ label: String) throws -> [Float] {
    try readVector(values.map { Double($0) }, count, label)
}

/// A vertex position of 1 to 4 Floats, checked as the Double one is.
func readPosition(_ values: [Float], _ label: String) throws -> [Float] {
    try readPosition(values.map { Double($0) }, label)
}

/// The number as the shared Rust message prints it: `NaN`, `inf`, `-inf`, or `1e39`.
/// Only those reach the message, since every other Double fits a Float.
func showNumber(_ value: Double) -> String {
    if value.isNaN { return "NaN" }
    if value.isInfinite { return value < 0 ? "-inf" : "inf" }
    return "\(value)".replacingOccurrences(of: "e+", with: "e")
}

/// A URL input for `http(s)`, otherwise a file path.
private func pathInput(_ path: String) -> TextureInputMobile {
    path.hasPrefix("http://") || path.hasPrefix("https://") ? .url(path) : .path(path)
}

/// Rust's default `TextureOptions` with an optional size, format and usage,
/// or `nil` when nothing differs, so the renderer applies its own defaults.
private func textureOptions(
    size: Size? = nil,
    format: TextureFormat? = nil,
    usage: TextureUsage? = nil
) -> TextureOptions? {
    if size == nil && format == nil && usage == nil {
        return nil
    }
    return TextureOptions(
        size: size,
        format: format ?? .rgba,
        sampler: SamplerOptions(repeatX: false, repeatY: false, smooth: true, compare: nil),
        mipmaps: true,
        usage: usage?.bits
    )
}
