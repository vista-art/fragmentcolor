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
    public static func directional(_ direction: [Double], _ color: [Double]) throws -> Light {
        return try Light.directional(
            direction: direction.map { Float($0) },
            color: color.map { Float($0) }
        )
    }

    public static func point(_ position: [Float], _ color: [Float]) throws -> Light {
        return try Light.point(position: position, color: color)
    }
    public static func point(_ position: [Double], _ color: [Double]) throws -> Light {
        return try Light.point(
            position: position.map { Float($0) },
            color: color.map { Float($0) }
        )
    }

    public static func spot(_ position: [Float], _ direction: [Float], _ color: [Float]) throws -> Light {
        return try Light.spot(position: position, direction: direction, color: color)
    }
    public static func spot(_ position: [Double], _ direction: [Double], _ color: [Double]) throws -> Light {
        return try Light.spot(
            position: position.map { Float($0) },
            direction: direction.map { Float($0) },
            color: color.map { Float($0) }
        )
    }

    // MARK: - Setters

    public func setColor(_ color: [Float]) throws -> Light {
        return try setColor(color: color)
    }
    public func setColor(_ color: [Double]) throws -> Light {
        return try setColor(color: color.map { Float($0) })
    }

    public func setIntensity(_ value: Float) -> Light {
        return setIntensity(value: value)
    }
    public func setIntensity(_ value: Double) -> Light {
        return setIntensity(value: Float(value))
    }

    public func setPosition(_ position: [Float]) throws -> Light {
        return try setPosition(position: position)
    }
    public func setPosition(_ position: [Double]) throws -> Light {
        return try setPosition(position: position.map { Float($0) })
    }

    public func setDirection(_ direction: [Float]) throws -> Light {
        return try setDirection(direction: direction)
    }
    public func setDirection(_ direction: [Double]) throws -> Light {
        return try setDirection(direction: direction.map { Float($0) })
    }

    public func setRange(_ value: Float) throws -> Light {
        return try setRange(value: value)
    }
    public func setRange(_ value: Double) throws -> Light {
        return try setRange(value: Float(value))
    }

    public func setConeAngles(_ innerRadians: Float, _ outerRadians: Float) throws -> Light {
        return try setConeAngles(innerRadians: innerRadians, outerRadians: outerRadians)
    }
    public func setConeAngles(_ innerRadians: Double, _ outerRadians: Double) throws -> Light {
        return try setConeAngles(
            innerRadians: Float(innerRadians),
            outerRadians: Float(outerRadians)
        )
    }
}
