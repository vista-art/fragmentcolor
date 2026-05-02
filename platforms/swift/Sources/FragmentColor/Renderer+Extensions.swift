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
}
