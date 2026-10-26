//
//  Model+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated Model API.
//  Matches the call shape used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

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
    /// The binding takes the same matrix flat, column after column.
    public func setTransform(_ columns: [[Float]]) throws {
        try setTransform(matrix: columns.flatMap { $0 })
    }
    @_disfavoredOverload
    public func setTransform(_ columns: [[Double]]) throws {
        try setTransform(matrix: columns.flatMap { $0 }.map { Float($0) })
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
