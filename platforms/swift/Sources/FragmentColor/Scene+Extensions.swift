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
}
