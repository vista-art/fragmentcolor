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
        try addVertex(v: Vertex(position: readPosition(position, "Mesh.addVertex: vertex")))
    }

    /// Append a single vertex from a Double array (converted to Float).
    @_disfavoredOverload
    public func addVertex(_ position: [Double]) throws {
        try addVertex(v: Vertex(position: readPosition(position, "Mesh.addVertex: vertex")))
    }

    /// Append a vertex at `position` with a `"uv"` attribute,
    /// as Rust's `(position, uv)` tuple does: 3 and 2 finite numbers.
    public func addVertex(_ position: [Float], _ uv: [Float]) throws {
        let position = try readVector(position, 3, "Mesh.addVertex: vertex: position")
        let uv = try readVector(uv, 2, "Mesh.addVertex: vertex: uv")
        addVertex(try Vertex(position: position).set("uv", uv))
    }

    @_disfavoredOverload
    public func addVertex(_ position: [Double], _ uv: [Double]) throws {
        let position = try readVector(position, 3, "Mesh.addVertex: vertex: position")
        let uv = try readVector(uv, 2, "Mesh.addVertex: vertex: uv")
        addVertex(try Vertex(position: position).set("uv", uv))
    }

    /// Append a vertex at `position` with `"uv"` and `"color"` attributes,
    /// as Rust's `(position, uv, color)` tuple does: 3, 2 and 4 finite numbers.
    public func addVertex(_ position: [Float], _ uv: [Float], _ color: [Float]) throws {
        let position = try readVector(position, 3, "Mesh.addVertex: vertex: position")
        let uv = try readVector(uv, 2, "Mesh.addVertex: vertex: uv")
        let color = try readVector(color, 4, "Mesh.addVertex: vertex: color")
        addVertex(try Vertex(position: position).set("uv", uv).set("color", color))
    }

    @_disfavoredOverload
    public func addVertex(_ position: [Double], _ uv: [Double], _ color: [Double]) throws {
        let position = try readVector(position, 3, "Mesh.addVertex: vertex: position")
        let uv = try readVector(uv, 2, "Mesh.addVertex: vertex: uv")
        let color = try readVector(color, 4, "Mesh.addVertex: vertex: color")
        addVertex(try Vertex(position: position).set("uv", uv).set("color", color))
    }

    /// Append multiple vertices from an array of float-array positions.
    public func addVertices(_ positions: [[Float]]) throws {
        try addVertices(vertices: readVertices(positions, "Mesh.addVertices"))
    }

    /// Append multiple vertices from an array of double-array positions.
    @_disfavoredOverload
    public func addVertices(_ positions: [[Double]]) throws {
        try addVertices(vertices: readVertices(positions, "Mesh.addVertices"))
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
        return Mesh.fromVertices(vertices: try readVertices(positions, "Mesh.fromVertices"))
    }

    /// Build a Mesh from an array of double-array positions.
    @_disfavoredOverload
    public static func fromVertices(_ positions: [[Double]]) throws -> Mesh {
        return Mesh.fromVertices(vertices: try readVertices(positions, "Mesh.fromVertices"))
    }
}

/// A vertex for each position, naming a failing one as `{label}: vertex {index}`.
private func readVertices(_ positions: [[Float]], _ label: String) throws -> [Vertex] {
    try positions.enumerated().map { index, position in
        try Vertex(position: readPosition(position, "\(label): vertex \(index)"))
    }
}

/// A vertex for each position, naming a failing one as `{label}: vertex {index}`.
private func readVertices(_ positions: [[Double]], _ label: String) throws -> [Vertex] {
    try positions.enumerated().map { index, position in
        try Vertex(position: readPosition(position, "\(label): vertex \(index)"))
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
    @_disfavoredOverload
    public convenience init(_ position: [Double]) throws {
        try self.init(position: readFloats(position, "Vertex: position"))
    }

    /// Unlabeled `Vertex.pbr([x, y, z])` matching the JS / Python spelling.
    public static func pbr(_ position: [Float]) throws -> Vertex {
        return try Vertex.pbr(position: position)
    }
    @_disfavoredOverload
    public static func pbr(_ position: [Double]) throws -> Vertex {
        return try Vertex.pbr(position: readFloats(position, "Vertex.pbr: position"))
    }

    // MARK: - Attributes

    // Each overload only says what kind of number it carries. The shader that
    // reads the attribute decides its type when the mesh attaches, so `7` fills
    // an f32, i32 or u32 input. A list holds 1 to 4 numbers, and any other
    // length throws.

    public func set(_ key: String, _ value: Float) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Double) throws -> Vertex {
        try set(key: key, value: [value])
    }

    public func set(_ key: String, _ value: Int) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Int32) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Int64) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: UInt) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: UInt32) throws -> Vertex {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ values: [Float]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Double]) throws -> Vertex {
        try set(key: key, value: values)
    }

    public func set(_ key: String, _ values: [Int]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Int32]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Int64]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [UInt]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [UInt32]) throws -> Vertex {
        try set(key: key, value: values.map { Double($0) })
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

    /// Append an instance from attribute lists, in the order written.
    /// A list holds 1 to 4 numbers, and any other length throws.
    public func addInstance(_ attributes: KeyValuePairs<String, [Double]>) throws {
        addInstance(instance: try attributes.reduce(Instance()) { try $0.set($1.key, $1.value) })
    }

    /// Append an instance from attribute lists, in the order written.
    public func addInstance(_ attributes: KeyValuePairs<String, [Float]>) throws {
        addInstance(instance: try attributes.reduce(Instance()) { try $0.set($1.key, $1.value) })
    }

    /// Append an instance from attribute lists, in the order written.
    public func addInstance(_ attributes: KeyValuePairs<String, [Int]>) throws {
        addInstance(instance: try attributes.reduce(Instance()) { try $0.set($1.key, $1.value) })
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
    @_disfavoredOverload
    public convenience init(_ min: [Double], _ max: [Double]) throws {
        try self.init(min: readFloats(min, "Quad: min"), max: readFloats(max, "Quad: max"))
    }
}

extension Instance {
    /// Create an empty Instance. Same as `Instance()` but matches the
    /// JS / Python `Instance.new()` call shape used in doc examples.
    public static func new() -> Instance {
        return Instance()
    }

    // MARK: - Attributes

    // Each overload only says what kind of number it carries. The shader that
    // reads the attribute decides its type when the mesh attaches, so `7` fills
    // an f32, i32 or u32 input. A list holds 1 to 4 numbers, and any other
    // length throws.

    public func set(_ key: String, _ value: Float) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Double) throws -> Instance {
        try set(key: key, value: [value])
    }

    public func set(_ key: String, _ value: Int) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Int32) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: Int64) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: UInt) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ value: UInt32) throws -> Instance {
        try set(key: key, value: [Double(value)])
    }

    public func set(_ key: String, _ values: [Float]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Double]) throws -> Instance {
        try set(key: key, value: values)
    }

    public func set(_ key: String, _ values: [Int]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Int32]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [Int64]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [UInt]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }

    public func set(_ key: String, _ values: [UInt32]) throws -> Instance {
        try set(key: key, value: values.map { Double($0) })
    }
}
