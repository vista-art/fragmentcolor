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

    /// Compute-pass constructor with an unlabeled name argument.
    public static func compute(_ name: String) -> Pass {
        return Pass.compute(name: name)
    }

    /// A pass holding `shader`, render or compute to match it (unlabeled overload).
    public static func fromShader(_ name: String, _ shader: Shader) -> Pass {
        return Pass.fromShader(name: name, shader: shader)
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
    // of pass.add(object: .model(model)). Each returns the pass, so calls chain.

    /// Attach a `Model` to the pass.
    @discardableResult
    public func add(_ model: Model) throws -> Pass {
        try self.add(object: .model(model))
    }

    /// Attach a `Camera` to the pass.
    @discardableResult
    public func add(_ camera: Camera) throws -> Pass {
        try self.add(object: .camera(camera))
    }

    /// Attach a `Light` to the pass.
    @discardableResult
    public func add(_ light: Light) throws -> Pass {
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

    /// Declare a `Scene`'s passes as dependencies of this pass.
    public func require(_ scene: Scene) throws {
        try self.require(deps: [.scene(scene)])
    }

    /// Declare multiple `Pass` objects as dependencies (in order).
    public func require(_ passes: [Pass]) throws {
        try self.require(deps: passes.map { .pass($0) })
    }

    /// Declare a heterogeneous list of renderables as dependencies.
    /// Each element is a `RenderableHandle` variant (`.shader`, `.pass`, `.mesh`, `.scene` or `.passes`).
    public func require(_ deps: [RenderableHandle]) throws {
        try self.require(deps: deps)
    }

    // MARK: - Targets

    /// Set the color attachment target for this pass.
    public func setTarget(_ target: TextureTarget) throws {
        try self.setTarget(target: .texture(target))
    }

    /// Render into a `Texture`, such as one from `Renderer.createTexture`.
    public func setTarget(_ texture: Texture) throws {
        try self.setTargetTexture(texture: texture)
    }

    /// Set the depth attachment target for this pass.
    public func setDepthTarget(_ target: TextureTarget) throws {
        try self.setDepthTarget(target: .texture(target))
    }

    /// Attach a depth `Texture` (created via `Renderer.createDepthTexture`)
    /// as the depth-stencil attachment. Mirrors the Kotlin
    /// `Pass.setDepthTarget(Texture)` extension.
    public func setDepthTarget(_ texture: Texture) throws {
        try self.setDepthTexture(texture: texture)
    }

    // MARK: - Clear color

    /// Set the clear color from `[r, g, b]` or `[r, g, b, a]`, each channel passed to the GPU as given.
    public func setClearColor(_ rgba: [Float]) throws {
        try self.setClearColor(color: rgba)
    }

    /// Set the clear color from `[r, g, b]` or `[r, g, b, a]` as doubles.
    public func setClearColor(_ rgba: [Double]) throws {
        try self.setClearColor(color: rgba.map { Float($0) })
    }

    // MARK: - Viewport

    /// Set the viewport (unlabeled overload). A width or height of 0 throws.
    public func setViewport(_ region: ScreenRegion) throws {
        try setViewport(region: region)
    }

    /// Set the viewport from `[(x, y), (width, height)]`, an origin and a size,
    /// as `ScreenRegion(origin, size)` reads them. A width or height of 0 throws.
    public func setViewport(_ originAndSize: [(UInt32, UInt32)]) throws {
        guard originAndSize.count == 2 else {
            throw FragmentColorError.Render("setViewport: expected [(x, y), (width, height)], got \(originAndSize.count) pairs")
        }
        try setViewport(region: ScreenRegion(originAndSize[0], originAndSize[1]))
    }

    /// Set the viewport from `[x, y, width, height]`, or `[width, height]` at the origin.
    /// A width or height of 0, or any other count of numbers, throws.
    public func setViewport(_ rect: [UInt32]) throws {
        switch rect.count {
        case 2: try setViewport(region: ScreenRegion([0, 0], rect))
        case 4: try setViewport(region: ScreenRegion(Array(rect[0..<2]), Array(rect[2..<4])))
        default: throw FragmentColorError.Render("setViewport: expected [width, height] or [x, y, width, height], got \(rect.count) numbers")
        }
    }

    // MARK: - Compute dispatch

    /// Set the compute dispatch grid size.
    public func setComputeDispatch(_ x: UInt32, _ y: UInt32, _ z: UInt32) {
        setComputeDispatch(x: x, y: y, z: z)
    }
}

extension ScreenRegion {
    /// A region at `origin` (x, y) that is `size` (width, height) large, as Rust's
    /// `ScreenRegion::new(origin, size)` reads them. A side of 0 throws, and so does
    /// a far edge past `UInt32.max`.
    public init(_ origin: (UInt32, UInt32), _ size: (UInt32, UInt32)) throws {
        guard size.0 > 0 && size.1 > 0 else {
            throw FragmentColorError.Render("ScreenRegion: a region needs a width and a height of at least 1, got \(size.0) by \(size.1)")
        }
        let (maxX, xOverflow) = origin.0.addingReportingOverflow(size.0)
        let (maxY, yOverflow) = origin.1.addingReportingOverflow(size.1)
        guard !xOverflow && !yOverflow else {
            throw FragmentColorError.Render("ScreenRegion: a region at \(origin.0), \(origin.1) sized \(size.0) by \(size.1) reaches past \(UInt32.max)")
        }
        self.init(minX: origin.0, minY: origin.1, maxX: maxX, maxY: maxY)
    }

    /// A region at `origin` `[x, y]` that is `size` `[width, height]` large.
    public init(_ origin: [UInt32], _ size: [UInt32]) throws {
        guard origin.count == 2 && size.count == 2 else {
            throw FragmentColorError.Render("ScreenRegion: origin and size hold 2 numbers each")
        }
        try self.init((origin[0], origin[1]), (size[0], size[1]))
    }
}
