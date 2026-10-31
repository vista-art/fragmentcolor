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
        throw FragmentColorError.Render("\(label) takes 4 columns of 4 numbers, got \(columns.count) columns")
    }
    for (index, column) in columns.enumerated() where column.count != 4 {
        throw FragmentColorError.Render("\(label): column \(index) takes 4 numbers, got \(column.count)")
    }
    return columns.flatMap { $0 }
}

extension Model {
    // MARK: - Constructors

    public convenience init(_ mesh: Mesh, _ material: Material) {
        self.init(mesh: mesh, material: material)
    }

    // MARK: - Setters

    public func setTransform(_ matrix: [Float]) throws {
        try setTransform(matrix: matrix)
    }
    @_disfavoredOverload
    public func setTransform(_ matrix: [Double]) throws {
        try setTransform(matrix: matrix.map { Float($0) })
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
        try rotate(axis: readFloats(axis, "Model.rotate: axis"), radians: Float(radians))
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
