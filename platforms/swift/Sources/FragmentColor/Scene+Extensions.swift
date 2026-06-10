// Idiomatic Swift wrappers on top of the uniffi-generated Scene API.
// The mobile binding takes a SceneObjectHandle enum (Model / Camera / Light)
// and dispatches internally; these overloads let callers pass the concrete
// types directly so example code reads scene.add(model) instead of
// scene.add(.model(model)).

extension Scene {
    /// Add a [Model] to the scene.
    public func add(_ model: Model) throws {
        try self.add(object: .model(model))
    }

    /// Add a [Camera] to the scene.
    public func add(_ camera: Camera) throws {
        try self.add(object: .camera(camera))
    }

    /// Add a [Light] to the scene.
    public func add(_ light: Light) throws {
        try self.add(object: .light(light))
    }

    /// Attach a [Pass] to the scene (unlabeled overload).
    public func addPass(_ pass: Pass) {
        addPass(pass: pass)
    }

    /// Set the scene ambient colour from a `[r, g, b]` array (unlabeled overload).
    public func ambient(_ color: [Float]) throws {
        try ambient(color: color)
    }
    public func ambient(_ color: [Double]) throws {
        try ambient(color: color.map { Float($0) })
    }

    /// `Scene.load("path/to/model.glb")` — unlabeled path overload matching
    /// the JS / Python spelling.
    public static func load(_ path: String) throws -> Scene {
        return try Scene.load(path: path)
    }

    // add_to overloads: target a Pass by index or name, passing the concrete
    // object type directly instead of wrapping it in PassTarget /
    // SceneObjectHandle.
    public func addTo(_ index: UInt64, _ model: Model) throws {
        try addTo(target: .index(index), object: .model(model))
    }
    public func addTo(_ index: UInt64, _ camera: Camera) throws {
        try addTo(target: .index(index), object: .camera(camera))
    }
    public func addTo(_ index: UInt64, _ light: Light) throws {
        try addTo(target: .index(index), object: .light(light))
    }
    public func addTo(_ name: String, _ model: Model) throws {
        try addTo(target: .name(name), object: .model(model))
    }
    public func addTo(_ name: String, _ camera: Camera) throws {
        try addTo(target: .name(name), object: .camera(camera))
    }
    public func addTo(_ name: String, _ light: Light) throws {
        try addTo(target: .name(name), object: .light(light))
    }
}
