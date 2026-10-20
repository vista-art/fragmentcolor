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

    // Each overload only says what kind of value it carries. Rust shapes it
    // to the uniform's declared WGSL type, so `3` fits an f32, i32 or u32
    // uniform and four numbers fit a vec4 or a mat2x2.

    public func set(_ key: String, _ value: Float) throws {
        try set(key: key, value: .number(Double(value)))
    }

    public func set(_ key: String, _ value: Double) throws {
        try set(key: key, value: .number(value))
    }

    public func set(_ key: String, _ value: Int) throws {
        try set(key: key, value: .number(Double(value)))
    }

    public func set(_ key: String, _ value: Int32) throws {
        try set(key: key, value: .number(Double(value)))
    }

    public func set(_ key: String, _ value: UInt32) throws {
        try set(key: key, value: .number(Double(value)))
    }

    public func set(_ key: String, _ value: Bool) throws {
        try set(key: key, value: .bool(value))
    }

    public func set(_ key: String, _ values: [Float]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    public func set(_ key: String, _ values: [Double]) throws {
        try set(key: key, value: .numbers(values))
    }

    public func set(_ key: String, _ values: [Int]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    public func set(_ key: String, _ values: [Int32]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    public func set(_ key: String, _ values: [UInt32]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    /// Raw bytes for a storage-buffer or push-constant root.
    public func set(_ key: String, _ bytes: Data) throws {
        try set(key: key, value: .bytes(bytes))
    }

    /// Bind a `Texture` created by a `Renderer` to a texture uniform.
    public func set(_ key: String, _ texture: Texture) throws {
        try set(key: key, value: .texture(texture.id()))
    }

    /// Get the value of a uniform by key (unlabeled overload).
    public func get(_ key: String) throws -> UniformData {
        try get(key: key)
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

    // MARK: - Async fetch factory

    /// Resolve a single URL, slug, file path, or raw WGSL source asynchronously
    /// and return a compiled `Shader`. This is the explicit async factory that
    /// mirrors `Shader.fetch(...)` on Web and Python.
    ///
    /// The underlying uniffi binding exposes `fetch` as an instance method
    /// (uniffi 0.31 does not support async constructors); this static wrapper
    /// creates a throw-away default instance to call through, then returns the
    /// freshly fetched shader.
    ///
    /// Usage:
    /// ```swift
    /// let shader = try await Shader.fetch("https://example.com/shader.wgsl")
    /// ```
    public static func fetch(_ input: String) async throws -> Shader {
        return try await Shader.default().fetch(input: input)
    }

    /// Resolve an array of parts (URLs, slugs, paths, raw WGSL) asynchronously,
    /// merge them, and return a compiled `Shader`.
    ///
    /// Usage:
    /// ```swift
    /// let shader = try await Shader.fetch(["sdf2d/circle", mySource])
    /// ```
    public static func fetch(_ parts: [String]) async throws -> Shader {
        return try await Shader.default().fetchCompose(parts: parts)
    }
}
