// Idiomatic Swift wrappers on top of the uniffi-generated Scene API.
// The mobile binding takes a SceneObjectHandle enum (Model / Camera / Light)
// and dispatches internally; these overloads let callers pass the concrete
// types directly so example code reads scene.add(model) instead of
// scene.add(.model(model)). Every mutator returns the scene, so calls chain.

import Foundation

extension Scene {
    /// Add a [Model] to the scene.
    @discardableResult
    public func add(_ model: Model) throws -> Scene {
        try self.add(object: .model(model))
    }

    /// Add a [Camera] to the scene.
    @discardableResult
    public func add(_ camera: Camera) throws -> Scene {
        try self.add(object: .camera(camera))
    }

    /// Add a [Light] to the scene.
    @discardableResult
    public func add(_ light: Light) throws -> Scene {
        try self.add(object: .light(light))
    }

    /// Attach a [Pass] to the scene (unlabeled overload).
    @discardableResult
    public func addPass(_ pass: Pass) -> Scene {
        addPass(pass: pass)
    }

    /// Set the scene ambient color from a `[r, g, b]` array (unlabeled overload).
    @discardableResult
    public func ambient(_ color: [Float]) throws -> Scene {
        try ambient(color: color)
    }

    @discardableResult
    @_disfavoredOverload
    public func ambient(_ color: [Double]) throws -> Scene {
        try ambient(color: readFloats(color, "Scene.ambient"))
    }

    /// `Scene.load("path/to/model.glb")` — unlabeled path overload matching
    /// the JS / Python spelling. `cameras` and `lights` decide whether the
    /// file's own cameras and lights load.
    public static func load(_ path: String, cameras: Bool = true, lights: Bool = true) throws -> Scene {
        return try Scene.load(path: path, cameras: cameras, lights: lights)
    }

    /// Load a scene from the bytes of a `.glb` file, as JS and Python do.
    public static func load(_ bytes: Data, cameras: Bool = true, lights: Bool = true) throws -> Scene {
        return try Scene.loadBytes(bytes: bytes, cameras: cameras, lights: lights)
    }

    // Pass-graph CRUD + injection: unlabeled overloads so example code reads
    // scene.removePass(pass) instead of scene.removePass(pass: pass).
    public func removePass(_ pass: Pass) -> Bool {
        return removePass(pass: pass)
    }
    /// The pass at `index`, or `nil` when there is none.
    public func getPass(_ index: UInt32) -> Pass? {
        return getPass(index: index)
    }
    public func findPass(_ name: String) -> Pass? {
        return findPass(name: name)
    }
    public func setPasses(_ passes: [Pass]) {
        setPasses(passes: passes)
    }
    @discardableResult
    public func setDefaultCamera(_ camera: Camera) -> Scene {
        setDefaultCamera(camera: camera)
    }
    @discardableResult
    public func setDefaultLight(_ light: Light) -> Scene {
        setDefaultLight(light: light)
    }

    // add_to overloads: target a Pass by index or name, passing the concrete
    // object type directly instead of wrapping it in PassTarget /
    // SceneObjectHandle.
    @discardableResult
    public func addTo(_ index: UInt32, _ model: Model) throws -> Scene {
        try addTo(target: .index(index), object: .model(model))
    }
    @discardableResult
    public func addTo(_ index: UInt32, _ camera: Camera) throws -> Scene {
        try addTo(target: .index(index), object: .camera(camera))
    }
    @discardableResult
    public func addTo(_ index: UInt32, _ light: Light) throws -> Scene {
        try addTo(target: .index(index), object: .light(light))
    }
    @discardableResult
    public func addTo(_ name: String, _ model: Model) throws -> Scene {
        try addTo(target: .name(name), object: .model(model))
    }
    @discardableResult
    public func addTo(_ name: String, _ camera: Camera) throws -> Scene {
        try addTo(target: .name(name), object: .camera(camera))
    }
    @discardableResult
    public func addTo(_ name: String, _ light: Light) throws -> Scene {
        try addTo(target: .name(name), object: .light(light))
    }
}
