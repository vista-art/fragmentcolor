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
    public func setTransform(_ matrix: [Double]) throws {
        try setTransform(matrix: matrix.map { Float($0) })
    }

    /// Nested 4×4 row form — flatten on the way through. The docs example
    /// reads `model.setTransform([[2,0,0,0], [0,2,0,0], ...])`; the binding
    /// takes a flat 16-float column-major matrix.
    public func setTransform(_ rows: [[Float]]) throws {
        try setTransform(matrix: rows.flatMap { $0 })
    }
    public func setTransform(_ rows: [[Double]]) throws {
        try setTransform(matrix: rows.flatMap { $0 }.map { Float($0) })
    }

    public func translate(_ offset: [Float]) throws {
        try translate(offset: offset)
    }
    public func translate(_ offset: [Double]) throws {
        try translate(offset: offset.map { Float($0) })
    }

    public func rotate(_ axis: [Float], _ radians: Float) throws {
        try rotate(axis: axis, radians: radians)
    }
    public func rotate(_ axis: [Double], _ radians: Double) throws {
        try rotate(axis: axis.map { Float($0) }, radians: Float(radians))
    }

    public func scale(_ factor: [Float]) throws {
        try scale(factor: factor)
    }
    public func scale(_ factor: [Double]) throws {
        try scale(factor: factor.map { Float($0) })
    }

    public func setVisible(_ visible: Bool) {
        setVisible(visible: visible)
    }
}
