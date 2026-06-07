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
}
