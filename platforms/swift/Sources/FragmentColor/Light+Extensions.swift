//
//  Light+Extensions.swift
//  FragmentColor
//
//  Unlabeled-argument overloads for the uniffi-generated Light API.
//  Matches the call shape used by the JS / Python bindings so doc examples
//  read identically across every platform.
//

import Foundation

extension Light {
    // MARK: - Constructors

    public static func directional(_ direction: [Float], _ color: [Float]) throws -> Light {
        return try Light.directional(direction: direction, color: color)
    }
    @_disfavoredOverload
    public static func directional(_ direction: [Double], _ color: [Double]) throws -> Light {
        return try Light.directional(
            direction: readFloats(direction, "Light.directional: direction"),
            color: readFloats(color, "Light.directional: color")
        )
    }

    public static func point(_ position: [Float], _ color: [Float]) throws -> Light {
        return try Light.point(position: position, color: color)
    }
    @_disfavoredOverload
    public static func point(_ position: [Double], _ color: [Double]) throws -> Light {
        return try Light.point(
            position: readFloats(position, "Light.point: position"),
            color: readFloats(color, "Light.point: color")
        )
    }

    public static func spot(_ position: [Float], _ direction: [Float], _ color: [Float]) throws -> Light {
        return try Light.spot(position: position, direction: direction, color: color)
    }
    @_disfavoredOverload
    public static func spot(_ position: [Double], _ direction: [Double], _ color: [Double]) throws -> Light {
        return try Light.spot(
            position: readFloats(position, "Light.spot: position"),
            direction: readFloats(direction, "Light.spot: direction"),
            color: readFloats(color, "Light.spot: color")
        )
    }

    // MARK: - Setters

    @discardableResult
    public func setColor(_ color: [Float]) throws -> Light {
        return try setColor(color: color)
    }
    @discardableResult
    @_disfavoredOverload
    public func setColor(_ color: [Double]) throws -> Light {
        return try setColor(color: readFloats(color, "Light.setColor"))
    }

    @discardableResult
    public func setIntensity(_ value: Float) -> Light {
        return setIntensity(value: value)
    }
    @discardableResult
    public func setIntensity(_ value: Double) -> Light {
        return setIntensity(value: Float(value))
    }

    @discardableResult
    public func setPosition(_ position: [Float]) throws -> Light {
        return try setPosition(position: position)
    }
    @discardableResult
    @_disfavoredOverload
    public func setPosition(_ position: [Double]) throws -> Light {
        return try setPosition(position: readFloats(position, "Light.setPosition"))
    }

    @discardableResult
    public func setDirection(_ direction: [Float]) throws -> Light {
        return try setDirection(direction: direction)
    }
    @discardableResult
    @_disfavoredOverload
    public func setDirection(_ direction: [Double]) throws -> Light {
        return try setDirection(direction: readFloats(direction, "Light.setDirection"))
    }

    @discardableResult
    public func setRange(_ value: Float) throws -> Light {
        return try setRange(value: value)
    }
    @discardableResult
    public func setRange(_ value: Double) throws -> Light {
        return try setRange(value: Float(value))
    }

    @discardableResult
    public func setConeAngles(_ innerRadians: Float, _ outerRadians: Float) throws -> Light {
        return try setConeAngles(innerRadians: innerRadians, outerRadians: outerRadians)
    }
    @discardableResult
    public func setConeAngles(_ innerRadians: Double, _ outerRadians: Double) throws -> Light {
        return try setConeAngles(
            innerRadians: Float(innerRadians),
            outerRadians: Float(outerRadians)
        )
    }
}
