//
//  Error+Extensions.swift
//  FragmentColor
//
//  The uniffi wrapper gives `FragmentColorError` a `description` from the
//  Rust `Display`, the error's message, but no conformance that uses it.
//  With `CustomStringConvertible`, string interpolation and the generated
//  `errorDescription`, and so `localizedDescription`, return that text.
//
//  Each case is a class Python raises, without its `Error` suffix, and `Other`
//  for the base class. `name` gives the class name, as a JavaScript error's
//  `name` does, and `message` the text every case carries.
//

import Foundation

extension FragmentColorError: CustomStringConvertible {}

extension FragmentColorError {
    /// The class Python raises this error as, and the `name` of the JavaScript
    /// `Error`, such as `"InvalidInputError"`, or `"FragmentColorError"` for `Other`.
    public var name: String {
        switch self {
        case .Other: return "FragmentColorError"
        case .InvalidInput: return "InvalidInputError"
        case .UnsupportedMipmapFormat: return "UnsupportedMipmapFormatError"
        case .MalformedImage: return "MalformedImageError"
        case .OccludedSurface: return "OccludedSurfaceError"
        case .Device: return "DeviceError"
        case .Limit: return "LimitError"
        case .Feature: return "FeatureError"
        case .Binding: return "BindingError"
        case .Entry: return "EntryError"
        case .SceneLoad: return "SceneLoadError"
        case .Light: return "LightError"
        case .Camera: return "CameraError"
        case .Material: return "MaterialError"
        }
    }

    /// The error's full text, the same `message` every binding gives it.
    public var message: String {
        switch self {
        case let .Other(message),
            let .InvalidInput(message),
            let .UnsupportedMipmapFormat(message),
            let .MalformedImage(message),
            let .OccludedSurface(message),
            let .SceneLoad(message),
            let .Light(message),
            let .Camera(message),
            let .Material(message):
            return message
        case let .Device(message, _, _, _),
            let .Limit(message, _, _, _, _),
            let .Feature(message, _, _),
            let .Binding(message, _, _, _),
            let .Entry(message, _):
            return message
        }
    }
}
