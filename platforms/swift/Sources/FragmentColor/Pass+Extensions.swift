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
    // MARK: - Convenience initializer

    /// Create a Pass with an unlabeled name argument (matches JS/Python spelling).
    public convenience init(_ name: String) {
        self.init(name: name)
    }

    // MARK: - Shader / Mesh (unlabeled overloads)

    /// Attach a shader to this pass (unlabeled overload).
    public func addShader(_ shader: Shader) {
        addShader(shader: shader)
    }

    /// Attach a mesh to this pass (unlabeled overload).
    public func addMesh(_ mesh: Mesh) throws {
        try addMesh(mesh: mesh)
    }

    // `addMeshToShader(mesh:shader:)` was removed in the v0.11.0 naming pass —
    // the body was a thin convenience over `shader.addMesh(mesh)?` that
    // ignored the receiver. The uniffi-generated method is gone, so this
    // wrapper went with it. Callers attach a mesh to a specific shader by
    // calling `shader.addMesh(mesh)` directly.

    // MARK: - Scene objects (add)
    //
    // The mobile binding takes a SceneObjectHandle enum (Model / Camera /
    // Light) and dispatches internally; these overloads let callers pass the
    // concrete types directly so example code reads pass.add(model) instead
    // of pass.add(object: .model(model)).

    /// Attach a `Model` to the pass.
    public func add(_ model: Model) throws {
        try self.add(object: .model(model))
    }

    /// Attach a `Camera` to the pass.
    public func add(_ camera: Camera) throws {
        try self.add(object: .camera(camera))
    }

    /// Attach a `Light` to the pass.
    public func add(_ light: Light) throws {
        try self.add(object: .light(light))
    }

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
    public func setTarget(_ target: MobileTextureTarget) throws {
        try self.setTarget(target: .texture(target))
    }

    /// Set the depth attachment target for this pass.
    public func setDepthTarget(_ target: MobileTextureTarget) throws {
        try self.setDepthTarget(target: .texture(target))
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

    // MARK: - Viewport

    /// Set the viewport from a `[(x, y), (width, height)]`-style tuple array.
    /// Accepts `[(minX, minY), (maxX, maxY)]` as two-element tuple array.
    public func setViewport(_ corners: [(UInt32, UInt32)]) {
        guard corners.count == 2 else { return }
        setViewport(region: ScreenRegion(
            minX: corners[0].0, minY: corners[0].1,
            maxX: corners[1].0, maxY: corners[1].1
        ))
    }

    /// Set the viewport from a 4-element Int array `[minX, minY, maxX, maxY]`.
    public func setViewport(_ rect: [Int]) {
        guard rect.count == 4 else { return }
        setViewport(region: ScreenRegion(
            minX: UInt32(rect[0]), minY: UInt32(rect[1]),
            maxX: UInt32(rect[2]), maxY: UInt32(rect[3])
        ))
    }

    // MARK: - Compute dispatch

    /// Set the compute dispatch grid size.
    public func setComputeDispatch(_ x: UInt32, _ y: UInt32, _ z: UInt32) {
        setComputeDispatch(x: x, y: y, z: z)
    }
}
