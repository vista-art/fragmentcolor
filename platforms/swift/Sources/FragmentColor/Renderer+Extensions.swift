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

    /// Build a `MobileWindowTarget` from a `CAMetalLayer`. The uniffi-generated
    /// `createTarget(metalLayerPtr:)` only accepts a raw pointer because
    /// uniffi cannot marshal `CAMetalLayer` directly; this overload does
    /// the pointer extraction so call sites read naturally.
    public func createTarget(layer: CAMetalLayer) throws -> MobileWindowTarget {
        let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
        return try self.createTarget(metalLayerPtr: ptr)
    }

    // MARK: - createTextureTarget convenience

    /// Create a headless texture target from a `[width, height]` array.
    public func createTextureTarget(_ size: [UInt32]) async throws -> MobileTextureTarget {
        guard size.count >= 2 else {
            throw FragmentColorError.Render("createTextureTarget: size array must have at least 2 elements")
        }
        return try await createTextureTarget(width: size[0], height: size[1])
    }

    /// Create a headless texture target from a `[Int]` array (convenient for literals).
    public func createTextureTarget(_ size: [Int]) async throws -> MobileTextureTarget {
        try await createTextureTarget(size.map { UInt32($0) })
    }

    // MARK: - createDepthTexture convenience

    /// Create a depth texture from a `[width, height]` array.
    public func createDepthTexture(_ size: [UInt32]) async throws -> Texture {
        guard size.count >= 2 else {
            throw FragmentColorError.Render("createDepthTexture: size array must have at least 2 elements")
        }
        return try await createDepthTexture(width: size[0], height: size[1])
    }

    /// Create a depth texture from a `[Int]` array.
    public func createDepthTexture(_ size: [Int]) async throws -> Texture {
        try await createDepthTexture(size.map { UInt32($0) })
    }

    // MARK: - createStorageTexture convenience

    /// Create a storage texture from a `([width, height], format)` tuple.
    public func createStorageTexture(_ sizeAndFormat: ([Int], TextureFormat)) async throws -> Texture {
        let sz = sizeAndFormat.0
        guard sz.count >= 2 else {
            throw FragmentColorError.Render("createStorageTexture: size array must have at least 2 elements")
        }
        return try await createStorageTexture(
            size: Size(width: UInt32(sz[0]), height: UInt32(sz[1]), depth: nil),
            format: sizeAndFormat.1,
            data: nil,
            usageBits: nil
        )
    }

    /// Create a storage texture from a `([width, height], format, bytes)` tuple.
    public func createStorageTexture(_ sizeFormatData: ([Int], TextureFormat, [UInt8])) async throws -> Texture {
        let sz = sizeFormatData.0
        guard sz.count >= 2 else {
            throw FragmentColorError.Render("createStorageTexture: size array must have at least 2 elements")
        }
        let data = Data(sizeFormatData.2)
        return try await createStorageTexture(
            size: Size(width: UInt32(sz[0]), height: UInt32(sz[1]), depth: nil),
            format: sizeFormatData.1,
            data: data,
            usageBits: nil
        )
    }

    // MARK: - createTexture convenience

    /// Create a texture from raw bytes + size: `([UInt8], [Int])` tuple.
    public func createTexture(_ bytesAndSize: ([UInt8], [Int])) async throws -> Texture {
        let data = Data(bytesAndSize.0)
        let sz = bytesAndSize.1
        let options: TextureOptions?
        if sz.count >= 2 {
            options = TextureOptions(
                size: Size(width: UInt32(sz[0]), height: UInt32(sz[1]), depth: nil),
                format: .rgba,
                sampler: SamplerOptions(repeatX: false, repeatY: false, smooth: true, compare: nil),
                mipmaps: false,
                usage: nil
            )
        } else {
            options = nil
        }
        return try await createTexture(input: .bytes(data), options: options)
    }

    /// Create a texture from a `[Int]` byte array + `[Int]` size tuple.
    public func createTexture(_ bytesAndSize: ([Int], [Int])) async throws -> Texture {
        try await createTexture((bytesAndSize.0.map { UInt8($0) }, bytesAndSize.1))
    }

    /// Create a texture from a path string (URL or file path).
    public func createTexture(_ path: String) async throws -> Texture {
        if path.hasPrefix("http://") || path.hasPrefix("https://") {
            return try await createTexture(input: .url(path), options: nil)
        }
        return try await createTexture(input: .path(path), options: nil)
    }

    /// Create a texture from a `Mipmap`.
    public func createTexture(_ chain: Mipmap) async throws -> Texture {
        return try await createTexture(input: .prepared(chain), options: nil)
    }

    // MARK: - unregisterTexture convenience

    /// Unregister a texture by its `TextureId` record.
    public func unregisterTexture(_ id: TextureId) throws {
        try unregisterTexture(textureId: id.id)
    }

    // MARK: - render (unlabeled overloads)

    /// Single overloaded `render(...)` family that matches the spelling used
    /// by the JavaScript and Python bindings. The uniffi layer exports one
    /// concrete `render(renderable:target:)` method that takes
    /// `RenderableHandle` + `TargetHandle` enums — these extensions wrap the
    /// native types into the matching variants invisibly so callers just
    /// write `try renderer.render(shader, target)` (or `pass`, `mesh`,
    /// `passList`).
    public func render(_ shader: Shader, _ target: MobileWindowTarget) throws {
        try self.render(renderable: .shader(shader), target: .window(target))
    }

    public func render(_ shader: Shader, _ target: MobileTextureTarget) throws {
        try self.render(renderable: .shader(shader), target: .texture(target))
    }

    public func render(_ pass: Pass, _ target: MobileWindowTarget) throws {
        try self.render(renderable: .pass(pass), target: .window(target))
    }

    public func render(_ pass: Pass, _ target: MobileTextureTarget) throws {
        try self.render(renderable: .pass(pass), target: .texture(target))
    }

    public func render(_ mesh: Mesh, _ target: MobileWindowTarget) throws {
        try self.render(renderable: .mesh(mesh), target: .window(target))
    }

    public func render(_ mesh: Mesh, _ target: MobileTextureTarget) throws {
        try self.render(renderable: .mesh(mesh), target: .texture(target))
    }

    public func render(_ passes: [Pass], _ target: MobileWindowTarget) throws {
        try self.render(renderable: .passes(passes), target: .window(target))
    }

    public func render(_ passes: [Pass], _ target: MobileTextureTarget) throws {
        try self.render(renderable: .passes(passes), target: .texture(target))
    }
}
