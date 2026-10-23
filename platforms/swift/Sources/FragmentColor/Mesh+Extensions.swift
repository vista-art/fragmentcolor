//
//  Mesh+Extensions.swift
//  FragmentColor
//
//  Convenience extensions for idiomatic Swift call sites.
//  The uniffi-generated Mesh API uses explicit labels (v:, vertices:) and
//  requires Vertex objects; these overloads accept raw Float arrays so
//  examples and user code read naturally.
//

import Foundation

extension Mesh {
    // MARK: - Convenience vertex construction

    /// Append a single vertex already constructed.
    public func addVertex(_ vertex: Vertex) {
        addVertex(v: vertex)
    }

    /// Append a single vertex from a position float array.
    /// Accepts 1 to 4 components.
    public func addVertex(_ position: [Float]) throws {
        try addVertex(v: Vertex(position: position))
    }

    /// Append a single vertex from a Double array (converted to Float).
    public func addVertex(_ position: [Double]) throws {
        try addVertex(v: Vertex(position: position.map { Float($0) }))
    }

    /// Append a vertex at `position` with a `"uv"` attribute, as Rust's `(position, uv)` tuple does.
    public func addVertex(_ position: [Float], _ uv: [Float]) throws {
        addVertex(try Vertex(position).set("uv", uv))
    }

    public func addVertex(_ position: [Double], _ uv: [Double]) throws {
        addVertex(try Vertex(position).set("uv", uv))
    }

    /// Append a vertex at `position` with `"uv"` and `"color"` attributes,
    /// as Rust's `(position, uv, color)` tuple does.
    public func addVertex(_ position: [Float], _ uv: [Float], _ color: [Float]) throws {
        addVertex(try Vertex(position).set("uv", uv).set("color", color))
    }

    public func addVertex(_ position: [Double], _ uv: [Double], _ color: [Double]) throws {
        addVertex(try Vertex(position).set("uv", uv).set("color", color))
    }

    /// Append multiple vertices from an array of float-array positions.
    public func addVertices(_ positions: [[Float]]) throws {
        try addVertices(vertices: positions.map { try Vertex(position: $0) })
    }

    /// Append multiple vertices from an array of double-array positions.
    public func addVertices(_ positions: [[Double]]) throws {
        try addVertices(vertices: positions.map { try Vertex(position: $0.map { Float($0) }) })
    }

    /// Append multiple already-constructed vertices (drops the `vertices:` label).
    public func addVertices(_ vertices: [Vertex]) {
        addVertices(vertices: vertices)
    }

    /// Explicit indices, dropping the `indices:` label.
    public func setIndices(_ indices: [UInt32]) {
        setIndices(indices: indices)
    }

    /// Build a Mesh from vertices already constructed (unlabeled overload).
    public static func fromVertices(_ vertices: [Vertex]) -> Mesh {
        return Mesh.fromVertices(vertices: vertices)
    }

    /// Build a Mesh from an array of float-array positions.
    public static func fromVertices(_ positions: [[Float]]) throws -> Mesh {
        let m = Mesh()
        try m.addVertices(positions)
        return m
    }

    /// Build a Mesh from an array of double-array positions.
    public static func fromVertices(_ positions: [[Double]]) throws -> Mesh {
        let m = Mesh()
        try m.addVertices(positions)
        return m
    }
}

extension Vertex {
    /// The attribute key of a vertex normal, `"normal"`.
    public static let NORMAL = "normal"
    /// The attribute key of a vertex tangent, `"tangent"`.
    public static let TANGENT = "tangent"
    /// The attribute key of the first texture coordinate set, `"uv0"`.
    public static let UV0 = "uv0"
    /// The attribute key of the second texture coordinate set, `"uv1"`.
    public static let UV1 = "uv1"
    /// The attribute key of the first vertex color, `"color0"`.
    public static let COLOR0 = "color0"
    /// The attribute key of the second vertex color, `"color1"`.
    public static let COLOR1 = "color1"

    /// Convenience: construct a Vertex from an unlabeled float array.
    public convenience init(_ position: [Float]) throws {
        try self.init(position: position)
    }

    /// Convenience: construct a Vertex from an unlabeled double array.
    public convenience init(_ position: [Double]) throws {
        try self.init(position: position.map { Float($0) })
    }

    /// Unlabeled `Vertex.pbr([x, y, z])` matching the JS / Python spelling.
    public static func pbr(_ position: [Float]) throws -> Vertex {
        return try Vertex.pbr(position: position)
    }
    public static func pbr(_ position: [Double]) throws -> Vertex {
        return try Vertex.pbr(position: position.map { Float($0) })
    }

    /// Set a vertex attribute with a Float array value (dispatches by length).
    /// An empty array, or one of more than 4 values, stops the program.
    public func set(_ key: String, _ value: [Float]) -> Vertex {
        return set(key: key, value: floatValue(value, "Vertex.set"))
    }

    /// Set a vertex attribute with a Double array value (dispatches by length).
    public func set(_ key: String, _ value: [Double]) -> Vertex {
        return set(key, value.map { Float($0) })
    }

    /// Set a vertex attribute with a Float scalar value.
    public func set(_ key: String, _ value: Float) -> Vertex {
        return set(key: key, value: .f32(value))
    }

    /// Set a vertex attribute with a Double scalar value.
    public func set(_ key: String, _ value: Double) -> Vertex {
        return set(key: key, value: .f32(Float(value)))
    }

    /// Set a vertex attribute with an integer literal, stored as an `f32`
    /// like every untyped number. Pass `.u32(_:)` or `.i32(_:)` for an integer attribute.
    public func set(_ key: String, _ value: Int) -> Vertex {
        return set(key: key, value: .f32(Float(value)))
    }

    /// Set a vertex attribute with a typed value, such as `.u32(7)` (unlabeled overload).
    public func set(_ key: String, _ value: VertexValue) -> Vertex {
        return set(key: key, value: value)
    }
}

extension Mesh {
    // MARK: - Convenience instance methods

    /// Append a single instance without an argument label.
    public func addInstance(_ instance: Instance) {
        addInstance(instance: instance)
    }

    /// Append the per-instance attributes of a vertex, as Rust's `Instance::from(vertex)` does.
    public func addInstance(_ vertex: Vertex) {
        addInstance(instance: vertex.createInstance())
    }

    /// Append an instance from attribute arrays, in the order written.
    public func addInstance(_ attributes: KeyValuePairs<String, [Float]>) {
        addInstance(instance: attributes.reduce(Instance()) { $0.set($1.key, $1.value) })
    }

    /// Append an instance from typed attribute values, in the order written.
    public func addInstance(_ attributes: KeyValuePairs<String, VertexValue>) {
        addInstance(instance: attributes.reduce(Instance()) { $0.set($1.key, $1.value) })
    }

    /// Append multiple instances without an argument label.
    public func addInstances(_ instances: [Instance]) {
        addInstances(instances: instances)
    }

    /// Append the per-instance attributes of each vertex.
    public func addInstances(_ vertices: [Vertex]) {
        addInstances(instances: vertices.map { $0.createInstance() })
    }

    /// Set the instance draw count without an argument label.
    public func setInstanceCount(_ n: UInt32) {
        setInstanceCount(n: n)
    }
}

extension Quad {
    /// Create a Quad from two unlabeled corner arrays (clip-space min, max).
    public convenience init(_ min: [Float], _ max: [Float]) throws {
        try self.init(min: min, max: max)
    }

    /// Create a Quad from unlabeled Double corner arrays.
    public convenience init(_ min: [Double], _ max: [Double]) throws {
        try self.init(min: min.map { Float($0) }, max: max.map { Float($0) })
    }
}

extension Instance {
    /// Create an empty Instance. Same as `Instance()` but matches the
    /// JS / Python `Instance.new()` call shape used in doc examples.
    public static func new() -> Instance {
        return Instance()
    }

    /// Set a per-instance attribute from a Float array (dispatches by length).
    /// An empty array, or one of more than 4 values, stops the program.
    public func set(_ key: String, _ value: [Float]) -> Instance {
        return set(key: key, value: floatValue(value, "Instance.set"))
    }

    /// Set a per-instance attribute from a Double array.
    public func set(_ key: String, _ value: [Double]) -> Instance {
        return set(key, value.map { Float($0) })
    }

    /// Set a per-instance attribute from a Float scalar.
    public func set(_ key: String, _ value: Float) -> Instance {
        return set(key: key, value: .f32(value))
    }

    /// Set a per-instance attribute from a Double scalar.
    public func set(_ key: String, _ value: Double) -> Instance {
        return set(key: key, value: .f32(Float(value)))
    }

    /// Set a per-instance attribute from an integer literal, stored as an `f32`
    /// like every untyped number. Pass `.u32(_:)` or `.i32(_:)` for an integer attribute.
    public func set(_ key: String, _ value: Int) -> Instance {
        return set(key: key, value: .f32(Float(value)))
    }

    /// Set a per-instance attribute with a typed value, such as `.u32(7)` (unlabeled overload).
    public func set(_ key: String, _ value: VertexValue) -> Instance {
        return set(key: key, value: value)
    }
}

/// The `f32` vertex value of 1 to 4 floats. Any other length stops the program.
private func floatValue(_ value: [Float], _ method: String) -> VertexValue {
    switch value.count {
    case 1: return .f32(value[0])
    case 2: return .f32x2(value)
    case 3: return .f32x3(value)
    case 4: return .f32x4(value)
    default: preconditionFailure("\(method): expected 1 to 4 numbers, got \(value.count)")
    }
}
