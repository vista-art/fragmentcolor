//
//  Renderer+Extensions.swift
//  FragmentColor
//
//  Idiomatic Swift wrappers on top of the uniffi-generated API.
//  Matches the struct/method names used by the JavaScript and Python bindings.
//

import Foundation
import QuartzCore

extension Renderer {
    /// Build a `WindowTarget` from a `CAMetalLayer`. Wraps the uniffi
    /// `createTargetIos(metalLayerPtr:)` entry point.
    public func createTarget(layer: CAMetalLayer) throws -> WindowTarget {
        let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
        return try self.createTargetIos(metalLayerPtr: ptr)
    }

    /// Headless texture target. Matches the JS / Python spelling.
    public func createTextureTarget(width: UInt32, height: UInt32) async throws -> TextureTarget {
        return try await self.createTextureTargetMobile(width: width, height: height)
    }

    /// Single `render(...)` overload that dispatches to the correct
    /// uniffi method based on the target type.
    public func render(_ shader: Shader, _ target: WindowTarget) throws {
        try self.renderShaderMobile(shader: shader, target: target)
    }

    public func render(_ shader: Shader, _ target: TextureTarget) throws {
        try self.renderShaderTextureMobile(shader: shader, target: target)
    }
}
