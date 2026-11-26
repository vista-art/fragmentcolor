//! The crate's errors: `FragmentColorError`, `ErrorKind`,
//! and `ErrorClass` and `Classify`, the class each binding raises an error as.

use thiserror::Error;

mod error_kind;
pub use error_kind::*;

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

/// A number as an error prints it: a large one in exponent form, such as `1e39`.
pub(crate) fn show_number(value: f64) -> String {
    if value.is_finite() && value.abs() >= 1e16 {
        format!("{value:e}")
    } else {
        value.to_string()
    }
}

/// The class of error each binding raises, named after the Python exception class.
/// Python raises that class, JavaScript throws an `Error` with that `name`, and Swift
/// and Kotlin throw the case of that name without its `Error` suffix, `Other` for the base.
#[cfg(any(python, wasm, mobile, test))]
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

/// Swift and Kotlin name each case in their own extension, `FragmentColorError.name`.
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

/// What every binding raises for an error: its class, its full message,
/// and the fields it carries, by the names each binding gives them.
#[cfg(any(python, wasm, mobile, test))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ErrorShape {
    pub(crate) class: ErrorClass,
    pub(crate) message: String,
    pub(crate) fields: Vec<(&'static str, crate::core::Field)>,
}

/// An error that knows the class every binding raises it as, and the fields it carries.
#[cfg(any(python, wasm, mobile, test))]
pub(crate) trait Classify: std::fmt::Display {
    /// The class Python raises this error as, the `name` of the JavaScript `Error`,
    /// and the case Swift and Kotlin throw.
    fn classify(&self) -> ErrorClass;

    /// The fields the error carries, by name. Most errors carry none.
    fn list_fields(&self) -> Vec<(&'static str, crate::core::Field)> {
        Vec::new()
    }

    /// The error each binding raises: this error's class, its text and its fields.
    /// An error that only passes another one on gives that one's shape.
    fn shape(&self) -> ErrorShape {
        ErrorShape {
            class: self.classify(),
            message: self.to_string(),
            fields: self.list_fields(),
        }
    }
}

#[cfg(any(python, wasm, mobile, test))]
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

    /// Each wrapped error prints as itself, so it raises as itself, fields and all.
    fn shape(&self) -> ErrorShape {
        match self {
            FragmentColorError::Shader(error) => error.shape(),
            FragmentColorError::Color(error) => error.shape(),
            FragmentColorError::Size(error) => error.shape(),
            FragmentColorError::Pass(error) => error.shape(),
            FragmentColorError::Renderer(error) => error.shape(),
            FragmentColorError::Init(error) => error.shape(),
            FragmentColorError::Display(error) => error.shape(),
            other => ErrorShape {
                class: other.classify(),
                message: other.to_string(),
                fields: Vec::new(),
            },
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
        use crate::MaterialError;
        use crate::PassError;
        use crate::ScreenRegionError;
        use crate::ShaderError;
        use crate::SurfaceError;
        use crate::scene::{LightError, ModelError, SceneLoadError};
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
            (
                Box::new(LightError::NotFinite {
                    label: text("Light::set_intensity: value"),
                    value: f32::NAN,
                }),
                "InvalidInputError",
            ),
            (Box::new(ModelError::ZeroAxis), "InvalidInputError"),
            (
                Box::new(MaterialError::UvSet {
                    call: "PbrMaterial::normal_map().uv".into(),
                    set: 2,
                }),
                "MaterialError",
            ),
            (
                Box::new(MaterialError::Texture(TextureError::InvalidInput(text(
                    "x",
                )))),
                "InvalidInputError",
            ),
            (Box::new(FragmentColorError::Renderer(limit)), "LimitError"),
        ];

        for (error, name) in cases {
            assert_eq!(error.classify().name(), name);
        }
    }

    // Story: an error that only passes another one on raises as that one, with its
    // text and its fields, so a texture's device error reached through a renderer or a
    // material carries its kind, label and detail in every binding.
    #[test]
    fn wrapped_errors_take_the_inner_shape() {
        use crate::core::Field;
        use crate::texture::TextureError;
        use crate::{InitializationError, MaterialError, RendererError};
        let device = || TextureError::DeviceError {
            kind: ErrorKind::Validation,
            label: "texture view".into(),
            detail: "x".into(),
        };
        let expected = ErrorShape {
            class: ErrorClass::Device,
            message: device().to_string(),
            fields: vec![
                ("kind", Field::Kind(ErrorKind::Validation)),
                ("label", Field::Text("texture view".into())),
                ("detail", Field::Text("x".into())),
            ],
        };
        let wrapped: Vec<Box<dyn Classify>> = vec![
            Box::new(device()),
            Box::new(RendererError::TextureError(device())),
            Box::new(RendererError::InitializationError(
                InitializationError::TextureError(device()),
            )),
            Box::new(MaterialError::Texture(device())),
            Box::new(FragmentColorError::Renderer(RendererError::TextureError(
                device(),
            ))),
        ];

        for error in wrapped {
            assert_eq!(error.shape(), expected);
        }

        let limit = RendererError::LimitError {
            item: "group 9".into(),
            limit: "max_bind_groups".into(),
            value: 10,
            max: 4,
        };
        let shape = limit.shape();
        assert_eq!(shape.class, ErrorClass::Limit);
        assert_eq!(shape.message, limit.to_string());
        assert_eq!(shape.fields, limit.list_fields());
    }
}
