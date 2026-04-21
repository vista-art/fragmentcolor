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
    /// Build a `WindowTarget` from a `CAMetalLayer`. The uniffi-generated
    /// `createTarget(metalLayerPtr:)` only accepts a raw pointer because
    /// uniffi cannot marshal `CAMetalLayer` directly; this overload does
    /// the pointer extraction so call sites read naturally.
    public func createTarget(layer: CAMetalLayer) throws -> WindowTarget {
        let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
        return try self.createTarget(metalLayerPtr: ptr)
    }

    /// Render a `Shader` into a `WindowTarget`. Single overloaded `render(...)`
    /// dispatch that matches the spelling used by the JavaScript and Python
    /// bindings. The uniffi layer exports one concrete method per
    /// (renderable × target) combination because it cannot handle
    /// `&impl Renderable` / `&impl Target` generics — see the module docs
    /// in `src/renderer/platform/mobile/mod.rs` for the rationale.
    public func render(_ shader: Shader, _ target: WindowTarget) throws {
        try self.renderShader(shader: shader, target: target)
    }

    public func render(_ shader: Shader, _ target: TextureTarget) throws {
        try self.renderShaderToTexture(shader: shader, target: target)
    }
}
