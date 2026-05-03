//
//  Shader+Extensions.swift
//  FragmentColor
//
//  Idiomatic Swift wrappers on top of the uniffi-generated `Shader` API.
//  Matches the call shapes used by the JavaScript and Python bindings so
//  cross-platform examples read the same on every platform.
//

import Foundation

extension Shader {
    // MARK: - Convenience initializers

    /// Build a Shader from a WGSL source string (unlabeled overload).
    public convenience init(_ source: String) throws {
        try self.init(source: source)
    }

    // MARK: - Registry

    /// `Shader.setRegistry(...)` matching the JS / Python static-method
    /// spelling. Uniffi has no static-method form on `uniffi::Object`, so
    /// the underlying binding is a constructor that returns a default
    /// `Shader`; this wrapper discards the dummy instance so callers see
    /// `Void`.
    public static func setRegistry(_ baseUrl: String) {
        _ = Shader.setRegistry(baseUrl: baseUrl)
    }

    // MARK: - Compose (multi-part)

    /// Build a Shader from an array of source strings / registry slugs.
    /// Matches the JS/Python `Shader.new([parts])` spelling.
    public static func new(_ parts: [String]) throws -> Shader {
        return try Shader.compose(parts: parts)
    }

    // MARK: - Mesh attachment

    /// Attach a mesh to this shader (unlabeled overload).
    public func addMesh(_ mesh: Mesh) throws {
        try addMesh(mesh: mesh)
    }

    /// Detach a mesh from this shader (unlabeled overload).
    public func removeMesh(_ mesh: Mesh) {
        removeMesh(mesh: mesh)
    }

    /// Detach multiple meshes from this shader (unlabeled overload).
    public func removeMeshes(_ meshes: [Mesh]) {
        removeMeshes(meshes: meshes)
    }

    /// Validate that a mesh is compatible with this shader's vertex layout.
    public func validateMesh(_ mesh: Mesh) throws {
        try validateMesh(mesh: mesh)
    }

    // MARK: - Uniform setters

    /// Set a uniform by key with a typed value. Dispatches to the right
    /// `UniformData` variant.
    public func set(_ key: String, _ value: Float) throws {
        try set(key: key, value: .float(value))
    }

    public func set(_ key: String, _ value: Int32) throws {
        try set(key: key, value: .int(value))
    }

    public func set(_ key: String, _ value: UInt32) throws {
        try set(key: key, value: .uInt(value))
    }

    public func set(_ key: String, _ value: Bool) throws {
        try set(key: key, value: .bool(value))
    }

    /// Float array — dispatches to Float / Vec2..4 / Mat3 / Mat4 by length.
    public func set(_ key: String, _ value: [Float]) throws {
        let uniform: UniformData
        switch value.count {
        case 1:  uniform = .float(value[0])
        case 2:  uniform = .vec2(value)
        case 3:  uniform = .vec3(value)
        case 4:  uniform = .vec4(value)
        case 9:  uniform = .mat3(value)
        case 16: uniform = .mat4(value)
        default:
            throw FragmentColorError.Shader("Unsupported float array length: \(value.count) (expected 1/2/3/4/9/16)")
        }
        try set(key: key, value: uniform)
    }

    /// Double array — converted to Float and dispatched by length.
    public func set(_ key: String, _ value: [Double]) throws {
        try set(key, value.map { Float($0) })
    }

    /// Int32 array — dispatches to Int / IVec2..4 by length.
    public func set(_ key: String, _ value: [Int32]) throws {
        let uniform: UniformData
        switch value.count {
        case 1: uniform = .int(value[0])
        case 2: uniform = .iVec2(value)
        case 3: uniform = .iVec3(value)
        case 4: uniform = .iVec4(value)
        default:
            throw FragmentColorError.Shader("Unsupported int array length: \(value.count) (expected 1/2/3/4)")
        }
        try set(key: key, value: uniform)
    }

    /// UInt32 array — dispatches to UInt / UVec2..4 by length.
    public func set(_ key: String, _ value: [UInt32]) throws {
        let uniform: UniformData
        switch value.count {
        case 1: uniform = .uInt(value[0])
        case 2: uniform = .uVec2(value)
        case 3: uniform = .uVec3(value)
        case 4: uniform = .uVec4(value)
        default:
            throw FragmentColorError.Shader("Unsupported uint array length: \(value.count) (expected 1/2/3/4)")
        }
        try set(key: key, value: uniform)
    }

    /// Get the value of a uniform by key (unlabeled overload).
    public func get(_ key: String) throws -> UniformData {
        try get(key: key)
    }

    /// Pass a `Texture` handle for a sampler/texture binding. Storage merges
    /// the user-supplied id with the shader-parsed metadata at set time, so
    /// the placeholder values for `dim` / `arrayed` / `class` / `sampled`
    /// here are overwritten by the real reflection data.
    public func set(_ key: String, _ texture: Texture) throws {
        let meta = TextureMeta(
            id: texture.id(),
            dim: .d2,
            arrayed: false,
            class: .sampled(kind: .float, multi: false),
            sampled: true
        )
        try set(key: key, value: .texture(meta))
    }

    // MARK: - fromVertex / fromMesh (unlabeled overloads)

    /// Build a Shader from a Vertex (unlabeled overload).
    public static func fromVertex(_ vertex: Vertex) -> Shader {
        return Shader.fromVertex(vertex: vertex)
    }

    /// Build a Shader from a Mesh (unlabeled overload).
    public static func fromMesh(_ mesh: Mesh) -> Shader {
        return Shader.fromMesh(mesh: mesh)
    }
}
