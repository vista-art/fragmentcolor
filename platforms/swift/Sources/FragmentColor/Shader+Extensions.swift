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

    /// Build a Shader from parts: WGSL sources, registry slugs, URLs or file paths.
    public convenience init(_ parts: [String]) throws {
        let shader = try Shader.compose(parts: parts)
        self.init(unsafeFromHandle: shader.uniffiCloneHandle())
    }

    // MARK: - Registry

    /// Point registry slugs at `baseUrl`, for every Shader in the process.
    /// Matches the JS / Python static-method spelling.
    public static func setRegistry(_ baseUrl: String) {
        _ = Shader.useRegistry(baseUrl: baseUrl)
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

    // MARK: - Pipeline state

    /// Set how the renderer treats this shader's alpha (unlabeled overload).
    @discardableResult
    public func alphaMode(_ mode: AlphaMode) -> Shader {
        return alphaMode(mode: mode)
    }

    /// Draw both faces, or cull back faces (unlabeled overload).
    @discardableResult
    public func doubleSided(_ value: Bool) -> Shader {
        return doubleSided(value: value)
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

    public func set(_ key: String, _ value: Int64) throws {
        try set(key: key, value: .number(Double(value)))
    }

    public func set(_ key: String, _ value: UInt) throws {
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

    public func set(_ key: String, _ values: [Int64]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    public func set(_ key: String, _ values: [UInt]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    public func set(_ key: String, _ values: [UInt32]) throws {
        try set(key: key, value: .numbers(values.map { Double($0) }))
    }

    /// A matrix as its columns, such as the 4x4 Rust's `Camera::view_proj` returns.
    public func set(_ key: String, _ columns: [[Float]]) throws {
        try set(key, columns.flatMap { $0 })
    }

    /// A matrix as its columns.
    public func set(_ key: String, _ columns: [[Double]]) throws {
        try set(key, columns.flatMap { $0 })
    }

    /// A matrix as its columns.
    public func set(_ key: String, _ columns: [[Int]]) throws {
        try set(key, columns.flatMap { $0 })
    }

    /// Raw bytes for a storage-buffer or push-constant root.
    public func set(_ key: String, _ bytes: Data) throws {
        try set(key: key, value: .bytes(bytes))
    }

    /// Bind a `Texture` created by a `Renderer` to a texture uniform.
    public func set(_ key: String, _ texture: Texture) throws {
        try set(key: key, value: .texture(texture.id()))
    }

    /// The current value of the uniform at `key`, as JavaScript and Python return it:
    /// a `Bool`, `UInt32`, `Int32` or `Float` for a scalar, `[Float]`, `[UInt32]` or `[Int32]`
    /// for a vector or a matrix, column by column, the `TextureId` a texture slot binds,
    /// a `SamplerInfo` for a sampler, and `Data` for a struct, array or buffer root.
    /// Narrow it with `as?`, as in `try shader.get("resolution") as? [Float]`.
    public func get(_ key: String) throws -> Any {
        switch try getData(key: key) {
        case let .bool(flag): return flag
        case let .uInt(value): return value
        case let .int(value): return value
        case let .float(value): return value
        case let .floats(values): return values
        case let .uInts(values): return values
        case let .ints(values): return values
        case let .texture(id): return id
        case let .sampler(info): return info
        case let .bytes(data): return data
        }
    }

    // MARK: - fromVertex / fromMesh (unlabeled overloads)

    /// Build a Shader from a Vertex (unlabeled overload).
    public static func fromVertex(_ vertex: Vertex) throws -> Shader {
        return try Shader.fromVertex(vertex: vertex)
    }

    /// Build a Shader from a Mesh (unlabeled overload).
    public static func fromMesh(_ mesh: Mesh) throws -> Shader {
        return try Shader.fromMesh(mesh: mesh)
    }

    // MARK: - Async fetch factory

    /// Resolve a single URL, slug, file path, or raw WGSL source asynchronously
    /// and return a compiled `Shader`. This is the explicit async factory that
    /// mirrors `Shader.fetch(...)` on Web and Python.
    ///
    /// The underlying uniffi binding exposes `fetch` as an instance method;
    /// this static wrapper creates a throw-away default instance to call
    /// through, then returns the freshly fetched shader.
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
