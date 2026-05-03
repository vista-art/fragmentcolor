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
    /// `Shader.setRegistry(...)` matching the JS / Python static-method
    /// spelling. Uniffi has no static-method form on `uniffi::Object`, so
    /// the underlying binding is a constructor that returns a default
    /// `Shader`; this wrapper discards the dummy instance so callers see
    /// `Void`.
    public static func setRegistry(_ baseUrl: String) {
        _ = Shader.setRegistry(baseUrl: baseUrl)
    }

    /// Set a uniform by key with a typed value. Dispatches to the right
    /// `UniformData` variant.
    public func set(_ key: String, _ value: Float) throws {
        try set(key: key, value: .float(value: value))
    }

    public func set(_ key: String, _ value: Int32) throws {
        try set(key: key, value: .int(value: value))
    }

    public func set(_ key: String, _ value: UInt32) throws {
        try set(key: key, value: .uInt(value: value))
    }

    public func set(_ key: String, _ value: Bool) throws {
        try set(key: key, value: .bool(value: value))
    }

    /// Float array — dispatches to Float / Vec2..4 / Mat3 / Mat4 by length.
    public func set(_ key: String, _ value: [Float]) throws {
        let uniform: UniformData
        switch value.count {
        case 1:  uniform = .float(value: value[0])
        case 2:  uniform = .vec2(value: value)
        case 3:  uniform = .vec3(value: value)
        case 4:  uniform = .vec4(value: value)
        case 9:  uniform = .mat3(value: value)
        case 16: uniform = .mat4(value: value)
        default:
            throw FragmentColorError.Shader(message: "Unsupported float array length: \(value.count) (expected 1/2/3/4/9/16)")
        }
        try set(key: key, value: uniform)
    }

    /// Int32 array — dispatches to Int / IVec2..4 by length.
    public func set(_ key: String, _ value: [Int32]) throws {
        let uniform: UniformData
        switch value.count {
        case 1: uniform = .int(value: value[0])
        case 2: uniform = .iVec2(value: value)
        case 3: uniform = .iVec3(value: value)
        case 4: uniform = .iVec4(value: value)
        default:
            throw FragmentColorError.Shader(message: "Unsupported int array length: \(value.count) (expected 1/2/3/4)")
        }
        try set(key: key, value: uniform)
    }

    /// UInt32 array — dispatches to UInt / UVec2..4 by length.
    public func set(_ key: String, _ value: [UInt32]) throws {
        let uniform: UniformData
        switch value.count {
        case 1: uniform = .uInt(value: value[0])
        case 2: uniform = .uVec2(value: value)
        case 3: uniform = .uVec3(value: value)
        case 4: uniform = .uVec4(value: value)
        default:
            throw FragmentColorError.Shader(message: "Unsupported uint array length: \(value.count) (expected 1/2/3/4)")
        }
        try set(key: key, value: uniform)
    }

    /// Pass a `Texture` handle for a sampler/texture binding. Storage merges
    /// the user-supplied id with the shader-parsed metadata at set time, so
    /// the placeholder values for `dim` / `arrayed` / `class` / `sampled`
    /// here are overwritten by the real reflection data.
    public func set(_ key: String, _ texture: Texture) throws {
        let meta = TextureMeta(
            id: texture.id().value,
            dim: .D2,
            arrayed: false,
            class: .sampled(kind: .float, multi: false),
            sampled: true
        )
        try set(key: key, value: .texture(value: meta))
    }
}
