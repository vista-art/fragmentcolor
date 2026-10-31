//! `FragmentColorError`, and `ErrorClass` and
//! `Classify`: the class each binding raises an error as.

use thiserror::Error;

pub(crate) mod platform;
#[cfg(any(python, wasm))]
pub use platform::*;

// Top-level catch-all error

#[derive(Error, Debug)]
pub enum FragmentColorError {
    #[error(transparent)]
    Shader(#[from] crate::ShaderError),
    #[error(transparent)]
    Color(#[from] crate::ColorError),
    #[error(transparent)]
    Size(#[from] crate::SizeError),
    #[error(transparent)]
    Pass(#[from] crate::PassError),
    #[error(transparent)]
    Renderer(#[from] crate::RendererError),
    #[error(transparent)]
    Init(#[from] crate::InitializationError),
    #[error(transparent)]
    Display(#[from] crate::DisplayError),
    #[cfg(not(wasm))]
    #[error("Network Request Error: {0}")]
    NetworkRequest(#[from] crate::NetworkError),
    #[cfg(wasm)]
    #[error("FragmentColor WASM Error: {0}")]
    Error(String),
}

/// The class of error each binding raises, named after the Python exception class.
/// Python raises that class, and JavaScript throws an `Error` with that `name`.
#[cfg(any(python, wasm, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorClass {
    Base,
    InvalidInput,
    UnsupportedMipmapFormat,
    MalformedImage,
    OccludedSurface,
    Device,
    Limit,
    Feature,
    Binding,
    Entry,
    SceneLoad,
    Light,
    Camera,
    Material,
}

#[cfg(any(python, wasm, test))]
impl ErrorClass {
    /// The class name, the same in Python and JavaScript.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ErrorClass::Base => "FragmentColorError",
            ErrorClass::InvalidInput => "InvalidInputError",
            ErrorClass::UnsupportedMipmapFormat => "UnsupportedMipmapFormatError",
            ErrorClass::MalformedImage => "MalformedImageError",
            ErrorClass::OccludedSurface => "OccludedSurfaceError",
            ErrorClass::Device => "DeviceError",
            ErrorClass::Limit => "LimitError",
            ErrorClass::Feature => "FeatureError",
            ErrorClass::Binding => "BindingError",
            ErrorClass::Entry => "EntryError",
            ErrorClass::SceneLoad => "SceneLoadError",
            ErrorClass::Light => "LightError",
            ErrorClass::Camera => "CameraError",
            ErrorClass::Material => "MaterialError",
        }
    }
}

/// An error that knows the class every binding raises it as.
#[cfg(any(python, wasm, test))]
pub(crate) trait Classify {
    /// The class Python raises this error as, and the `name` of the JavaScript `Error`.
    fn classify(&self) -> ErrorClass;
}

#[cfg(any(python, wasm, test))]
impl Classify for FragmentColorError {
    fn classify(&self) -> ErrorClass {
        match self {
            FragmentColorError::Shader(error) => error.classify(),
            FragmentColorError::Color(error) => error.classify(),
            FragmentColorError::Size(error) => error.classify(),
            FragmentColorError::Pass(error) => error.classify(),
            FragmentColorError::Renderer(error) => error.classify(),
            FragmentColorError::Init(error) => error.classify(),
            FragmentColorError::Display(error) => error.classify(),
            #[cfg(not(wasm))]
            FragmentColorError::NetworkRequest(_) => ErrorClass::Base,
            #[cfg(wasm)]
            FragmentColorError::Error(_) => ErrorClass::Base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Top-level error enum wraps inner error kinds via From and displays sensible messages.
    #[test]
    fn wraps_inner_errors_and_displays_messages() {
        // Arrange / Act
        let e1: FragmentColorError = crate::ShaderError::UniformNotFound("u".into()).into();
        let e2: FragmentColorError =
            crate::ColorError::Invalid("a color takes 3 or 4 channels".into()).into();
        let e3: FragmentColorError =
            crate::SizeError::Invalid("a size takes 2 or 3 numbers".into()).into();
        let e6: FragmentColorError = crate::RendererError::NoContext.into();
        let e7: FragmentColorError = crate::InitializationError::Error("init".into()).into();
        let e8: FragmentColorError = crate::DisplayError::Error("disp".into()).into();

        #[cfg(not(wasm))]
        let e9: FragmentColorError = {
            let err = crate::NetworkError("connection refused".into());
            err.into()
        };

        // Assert
        assert!(e1.to_string().contains("Uniform not found"));
        assert_eq!(e2.to_string(), "a color takes 3 or 4 channels");
        assert_eq!(e3.to_string(), "a size takes 2 or 3 numbers");
        assert!(e6.to_string().contains("Context not initialized"));
        assert!(e7.to_string().contains("Initialization error"));
        assert!(e8.to_string().contains("Display Error"));
        #[cfg(not(wasm))]
        assert!(e9.to_string().contains("Network"));
    }

    // Story: each error takes the class Python raises it as, which is also the `name` of the
    // Error JavaScript throws, and an error that wraps another takes the class of what it wraps.
    #[test]
    fn errors_take_the_python_class() {
        use crate::ColorError;
        use crate::PassError;
        use crate::ScreenRegionError;
        use crate::ShaderError;
        use crate::SurfaceError;
        use crate::scene::{LightError, SceneLoadError};
        use crate::texture::TextureError;
        use crate::{InitializationError, RendererError};
        let text = |value: &str| value.to_string();
        let limit = RendererError::LimitError {
            item: text("group 9"),
            limit: text("max_bind_groups"),
            value: 10,
            max: 4,
        };
        let cases: Vec<(Box<dyn Classify>, &str)> = vec![
            (
                Box::new(ShaderError::ParseError(text("x"))),
                "FragmentColorError",
            ),
            (Box::new(RendererError::NoContext), "FragmentColorError"),
            (
                Box::new(RendererError::MeshError(crate::MeshError::Busy(4))),
                "FragmentColorError",
            ),
            (
                Box::new(RendererError::SurfaceError(SurfaceError::Lost)),
                "FragmentColorError",
            ),
            (
                Box::new(RendererError::SurfaceError(SurfaceError::Occluded)),
                "OccludedSurfaceError",
            ),
            (
                Box::new(RendererError::DeviceError {
                    kind: crate::ErrorKind::Validation,
                    label: text("frame"),
                    detail: text("x"),
                }),
                "DeviceError",
            ),
            (
                Box::new(RendererError::FeatureError {
                    item: text("the shader"),
                    feature: text("x"),
                }),
                "FeatureError",
            ),
            (
                Box::new(RendererError::BindingError {
                    uniform: text("image"),
                    expected: text("a texture"),
                    found: text("none"),
                }),
                "BindingError",
            ),
            (
                Box::new(RendererError::EntryError {
                    stage: text("@vertex"),
                }),
                "EntryError",
            ),
            (
                Box::new(RendererError::TextureError(TextureError::InvalidInput(
                    text("x"),
                ))),
                "InvalidInputError",
            ),
            (
                Box::new(RendererError::PassError(PassError::Viewport(
                    ScreenRegionError::Empty {
                        width: 0,
                        height: 4,
                    },
                ))),
                "InvalidInputError",
            ),
            (
                Box::new(RendererError::InitializationError(
                    InitializationError::TextureError(TextureError::MalformedImageError(text("x"))),
                )),
                "MalformedImageError",
            ),
            (
                Box::new(TextureError::UnsupportedMipmapFormat {
                    format: crate::TextureFormat::R8Unorm,
                }),
                "UnsupportedMipmapFormatError",
            ),
            (
                Box::new(ColorError::Invalid(text("x"))),
                "InvalidInputError",
            ),
            (
                Box::new(SceneLoadError::Invalid(text("x"))),
                "SceneLoadError",
            ),
            (Box::new(LightError::NegativeRange(-1.0)), "LightError"),
            (Box::new(FragmentColorError::Renderer(limit)), "LimitError"),
        ];

        for (error, name) in cases {
            assert_eq!(error.classify().name(), name);
        }
    }
}
