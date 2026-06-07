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
    public func addVertex(_ vertex: Vertex) throws {
        try addVertex(v: vertex)
    }

    /// Append a single vertex from a position float array.
    /// Accepts 2, 3, or 4 components.
    public func addVertex(_ position: [Float]) throws {
        try addVertex(v: Vertex(position: position))
    }

    /// Append a single vertex from a Double array (converted to Float).
    public func addVertex(_ position: [Double]) throws {
        try addVertex(v: Vertex(position: position.map { Float($0) }))
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
    public func addVertices(_ vertices: [Vertex]) throws {
        try addVertices(vertices: vertices)
    }

    /// Explicit indices, dropping the `indices:` label.
    public func setIndices(_ indices: [UInt32]) {
        setIndices(indices: indices)
    }
    /// Explicit indices from a `[Int]` literal (convenient for example code).
    public func setIndices(_ indices: [Int]) {
        setIndices(indices: indices.map { UInt32($0) })
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
    public func set(_ key: String, _ value: [Float]) -> Vertex {
        let vv: VertexValue
        switch value.count {
        case 1:  vv = .f32(value[0])
        case 2:  vv = .f32x2(value)
        case 3:  vv = .f32x3(value)
        case 4:  vv = .f32x4(value)
        default: vv = .f32x4(Array(value.prefix(4)))
        }
        return set(key: key, value: vv)
    }

    /// Set a vertex attribute with a Double array value (dispatches by length).
    public func set(_ key: String, _ value: [Double]) -> Vertex {
        return set(key, value.map { Float($0) })
    }

    /// Set a vertex attribute with a Float scalar value.
    public func set(_ key: String, _ value: Float) -> Vertex {
        return set(key: key, value: .f32(value))
    }
}

extension Mesh {
    // MARK: - Convenience instance methods

    /// Append a single instance without an argument label.
    public func addInstance(_ instance: Instance) {
        addInstance(instance: instance)
    }

    /// Append multiple instances without an argument label.
    public func addInstances(_ instances: [Instance]) {
        addInstances(instances: instances)
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
    @discardableResult
    public func set(_ key: String, _ value: [Float]) -> Instance {
        let vv: VertexValue
        switch value.count {
        case 1:  vv = .f32(value[0])
        case 2:  vv = .f32x2(value)
        case 3:  vv = .f32x3(value)
        case 4:  vv = .f32x4(value)
        default: vv = .f32x4(Array(value.prefix(4)))
        }
        return set(key: key, value: vv)
    }

    /// Set a per-instance attribute from a Double array.
    @discardableResult
    public func set(_ key: String, _ value: [Double]) -> Instance {
        return set(key, value.map { Float($0) })
    }
}
