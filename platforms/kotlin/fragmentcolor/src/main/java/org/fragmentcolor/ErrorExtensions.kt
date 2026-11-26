package org.fragmentcolor

// Each FragmentColorException case is a class Python raises, without its `Error`
// suffix, and `Other` for the base class. Its `message` is the error's text, and
// `name` gives the class name, as a JavaScript error's `name` does.

/**
 * The class Python raises this error as, and the `name` of the JavaScript `Error`,
 * such as `"InvalidInputError"`, or `"FragmentColorError"` for [FragmentColorException.Other].
 */
val FragmentColorException.name: String
    get() = when (this) {
        is FragmentColorException.Other -> "FragmentColorError"
        is FragmentColorException.InvalidInput -> "InvalidInputError"
        is FragmentColorException.UnsupportedMipmapFormat -> "UnsupportedMipmapFormatError"
        is FragmentColorException.MalformedImage -> "MalformedImageError"
        is FragmentColorException.OccludedSurface -> "OccludedSurfaceError"
        is FragmentColorException.Device -> "DeviceError"
        is FragmentColorException.Limit -> "LimitError"
        is FragmentColorException.Feature -> "FeatureError"
        is FragmentColorException.Binding -> "BindingError"
        is FragmentColorException.Entry -> "EntryError"
        is FragmentColorException.SceneLoad -> "SceneLoadError"
        is FragmentColorException.Light -> "LightError"
        is FragmentColorException.Camera -> "CameraError"
        is FragmentColorException.Material -> "MaterialError"
    }
