//
//  Pass+Extensions.swift
//  FragmentColor
//
//  Idiomatic Swift wrappers on top of the uniffi-generated `Pass` API.
//  Provides natural overloads so callers never have to construct
//  `RenderableHandle` or `TargetHandle` variants by hand.
//

import Foundation

extension Pass {
    // MARK: - Dependencies (require)

    /// Declare a `Shader` as a dependency of this pass.
    public func require(_ shader: Shader) throws {
        try self.require(deps: [.shader(shader)])
    }

    /// Declare a `Pass` as a dependency of this pass.
    public func require(_ pass: Pass) throws {
        try self.require(deps: [.pass(pass)])
    }

    /// Declare a `Mesh` as a dependency of this pass.
    public func require(_ mesh: Mesh) throws {
        try self.require(deps: [.mesh(mesh)])
    }

    /// Declare multiple `Pass` objects as dependencies (in order).
    public func require(_ passes: [Pass]) throws {
        try self.require(deps: passes.map { .pass($0) })
    }

    /// Declare a heterogeneous list of renderables as dependencies.
    /// Each element is a `RenderableHandle` variant (`.shader`, `.pass`, `.mesh`, or `.passes`).
    public func require(_ deps: [RenderableHandle]) throws {
        try self.require(deps: deps)
    }

    // MARK: - Targets

    /// Set the colour attachment target for this pass.
    public func addTarget(_ target: TextureTarget) throws {
        try self.addTarget(target: .texture(target))
    }

    /// Set the depth attachment target for this pass.
    public func addDepthTarget(_ target: TextureTarget) throws {
        try self.addDepthTarget(target: .texture(target))
    }

    // MARK: - Clear colour

    /// Set the clear colour as separate RGBA components (0..1 linear space).
    public func setClearColor(r: Float, g: Float, b: Float, a: Float = 1.0) throws {
        try self.setClearColor(color: [r, g, b, a])
    }

    /// Set the clear colour from a 3- or 4-element float array (`[r, g, b]` or `[r, g, b, a]`).
    public func setClearColor(_ rgba: [Float]) throws {
        try self.setClearColor(color: rgba)
    }
}
