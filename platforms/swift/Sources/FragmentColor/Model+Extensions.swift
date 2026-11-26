//
//  Model+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated Model API.
//  Matches the call shape used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

/// A 4×4 matrix from its four columns, flat, column after column.
/// Throws unless it holds 4 columns of 4 numbers, naming the column that doesn't.
private func flattenColumns(_ columns: [[Float]], _ label: String) throws -> [Float] {
    guard columns.count == 4 else {
        throw FragmentColorError.InvalidInput(message: "\(label) takes 4 columns of 4 numbers, got \(columns.count) columns")
    }
    for (index, column) in columns.enumerated() where column.count != 4 {
        throw FragmentColorError.InvalidInput(message: "\(label): column \(index) takes 4 numbers, got \(column.count)")
    }
    return columns.flatMap { $0 }
}

/// What a Model draws with: a PbrMaterial, an UnlitMaterial, a ToonMaterial or a Shader.
/// `Model(mesh, material)` takes one, and `model.material()` returns it, which `as?` narrows,
/// as in `model.material() as? PbrMaterial`. Only those four types conform.
public protocol Material: AnyObject {
    /// The material as the generated `Model` initializer takes it.
    var materialHandle: MaterialHandle { get }
}

extension PbrMaterial: Material {
    public var materialHandle: MaterialHandle { .pbr(self) }
}

extension UnlitMaterial: Material {
    public var materialHandle: MaterialHandle { .unlit(self) }
}

extension ToonMaterial: Material {
    public var materialHandle: MaterialHandle { .toon(self) }
}

extension Shader: Material {
    public var materialHandle: MaterialHandle { .custom(self) }
}

extension Model {
    // MARK: - Constructors

    /// A Model that draws `mesh` with a PbrMaterial, an UnlitMaterial, a ToonMaterial or a Shader,
    /// including what another Model's `material()` returned.
    public convenience init(_ mesh: Mesh, _ material: any Material) {
        self.init(mesh: mesh, material: material.materialHandle)
    }

    // MARK: - Getters

    /// What the Model draws with: the PbrMaterial, UnlitMaterial, ToonMaterial or Shader itself,
    /// as JavaScript and Python return it. Narrow it with `as?`, as in `model.material() as? PbrMaterial`.
    public func material() -> any Material {
        switch materialHandle() {
        case .pbr(let material): return material
        case .unlit(let material): return material
        case .toon(let material): return material
        case .custom(let shader): return shader
        }
    }

    // MARK: - Setters

    public func setTransform(_ matrix: [Float]) throws {
        try setTransform(matrix: matrix)
    }
    @_disfavoredOverload
    public func setTransform(_ matrix: [Double]) throws {
        try setTransform(matrix: readFloats(matrix, "Model.setTransform"))
    }

    /// A 4×4 matrix as its four columns, as Rust's `[[f32; 4]; 4]` holds it.
    /// Throws unless it holds 4 columns of 4 numbers, naming the column that doesn't.
    public func setTransform(_ columns: [[Float]]) throws {
        try setTransform(matrix: flattenColumns(columns, "Model.setTransform"))
    }
    @_disfavoredOverload
    public func setTransform(_ columns: [[Double]]) throws {
        let floats = try columns.enumerated().map { index, column in
            try readFloats(column, "Model.setTransform: column \(index)")
        }
        try setTransform(matrix: flattenColumns(floats, "Model.setTransform"))
    }

    public func translate(_ offset: [Float]) throws {
        try translate(offset: offset)
    }
    @_disfavoredOverload
    public func translate(_ offset: [Double]) throws {
        try translate(offset: readFloats(offset, "Model.translate"))
    }

    public func rotate(_ axis: [Float], _ radians: Float) throws {
        try rotate(axis: axis, radians: radians)
    }
    @_disfavoredOverload
    public func rotate(_ axis: [Double], _ radians: Double) throws {
        try rotate(
            axis: readFloats(axis, "Model.rotate: axis"),
            radians: readFinite(radians, "Model.rotate: radians")
        )
    }

    public func scale(_ factor: [Float]) throws {
        try scale(factor: factor)
    }
    @_disfavoredOverload
    public func scale(_ factor: [Double]) throws {
        try scale(factor: readFloats(factor, "Model.scale"))
    }

    public func setVisible(_ visible: Bool) {
        setVisible(visible: visible)
    }
}
