//
//  Camera+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated Camera API.
//  Matches the call shape used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

extension Camera {
    // MARK: - Constructors

    public static func perspective(
        _ fovyRadians: Float,
        _ aspect: Float,
        _ near: Float,
        _ far: Float
    ) throws -> Camera {
        return try Camera.perspective(fovyRadians: fovyRadians, aspect: aspect, near: near, far: far)
    }

    public static func perspective(
        _ fovyRadians: Double,
        _ aspect: Double,
        _ near: Double,
        _ far: Double
    ) throws -> Camera {
        return try Camera.perspective(
            fovyRadians: Float(fovyRadians),
            aspect: Float(aspect),
            near: Float(near),
            far: Float(far)
        )
    }

    public static func orthographic(
        _ left: Float,
        _ right: Float,
        _ bottom: Float,
        _ top: Float,
        _ near: Float,
        _ far: Float
    ) throws -> Camera {
        return try Camera.orthographic(
            left: left, right: right, bottom: bottom, top: top, near: near, far: far
        )
    }

    public static func orthographic(
        _ left: Double,
        _ right: Double,
        _ bottom: Double,
        _ top: Double,
        _ near: Double,
        _ far: Double
    ) throws -> Camera {
        return try Camera.orthographic(
            left: Float(left),
            right: Float(right),
            bottom: Float(bottom),
            top: Float(top),
            near: Float(near),
            far: Float(far)
        )
    }

    // MARK: - Methods

    @discardableResult
    public func lookAt(_ position: [Float], _ target: [Float], _ up: [Float]) throws -> Camera {
        return try lookAt(position: position, target: target, up: up)
    }
    @discardableResult
    @_disfavoredOverload
    public func lookAt(_ position: [Double], _ target: [Double], _ up: [Double]) throws -> Camera {
        return try lookAt(
            position: readFloats(position, "Camera.lookAt: position"),
            target: readFloats(target, "Camera.lookAt: target"),
            up: readFloats(up, "Camera.lookAt: up")
        )
    }

    @discardableResult
    public func setAspect(_ aspect: Float) throws -> Camera {
        return try setAspect(aspect: aspect)
    }
    @discardableResult
    public func setAspect(_ aspect: Double) throws -> Camera {
        return try setAspect(aspect: Float(aspect))
    }
}
