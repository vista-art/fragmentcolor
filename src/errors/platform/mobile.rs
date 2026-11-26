//! The uniffi `FragmentColorError` that Swift and Kotlin throw.

use crate::core::Field;
use crate::errors::{Classify, ErrorClass, ErrorShape};
use crate::scene::{CameraError, LightError, ModelError, SceneLoadError};
use crate::texture::TextureError;
use crate::{ColorError, ErrorKind, InitializationError, MaterialError, MeshError, PassError};
use crate::{RendererError, ScreenRegionError, ShaderError, SizeError};

/// Mobile-facing error type, one typed error for Swift and Kotlin to match on.
///
/// Each case is a class Python raises, by the same name without its `Error` suffix,
/// such as `InvalidInput` for `InvalidInputError`, and `Other` for an error that
/// Python raises as the base `FragmentColorError`. Every case carries the error's
/// full `message`, and a case with fields carries them under the names Python and
/// JavaScript give them. The exported `Display` gives Swift's `description` and
/// `localizedDescription`, and Kotlin's `toString()`, the message.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi::export(Display)]
pub enum FragmentColorError {
    /// An error with no class of its own: Python's base `FragmentColorError`.
    #[error("{message}")]
    Other { message: String },
    /// A value the call can't take, such as a negative size or bytes that don't match their texture.
    #[error("{message}")]
    InvalidInput { message: String },
    /// A texture format the CPU mipmap builder can't fill.
    #[error("{message}")]
    UnsupportedMipmapFormat { message: String },
    /// Image bytes that can't be decoded.
    #[error("{message}")]
    MalformedImage { message: String },
    /// A window that is hidden or minimized, so render skips the frame.
    #[error("{message}")]
    OccludedSurface { message: String },
    /// A GPU error an error scope caught, with its kind, the label of the step
    /// that raised it and the device's own detail.
    #[error("{message}")]
    Device {
        message: String,
        kind: ErrorKind,
        label: String,
        detail: String,
    },
    /// Something in a shader or mesh past one of the device's limits.
    #[error("{message}")]
    Limit {
        message: String,
        item: String,
        limit: String,
        value: u64,
        max: u64,
    },
    /// Something in a shader that needs a feature the device lacks.
    #[error("{message}")]
    Feature {
        message: String,
        item: String,
        feature: String,
    },
    /// A texture uniform whose slot can't take its texture, or that was never set.
    #[error("{message}")]
    Binding {
        message: String,
        uniform: String,
        expected: String,
        found: String,
    },
    /// A shader without the entry point its pass runs.
    #[error("{message}")]
    Entry { message: String, stage: String },
    /// A scene file that fails to load.
    #[error("{message}")]
    SceneLoad { message: String },
    /// A light setter that doesn't fit the light's kind.
    #[error("{message}")]
    Light { message: String },
    /// Camera parameters that leave no usable view.
    #[error("{message}")]
    Camera { message: String },
    /// A material setter value the material can't use.
    #[error("{message}")]
    Material { message: String },
}

impl FragmentColorError {
    /// An `Other` error, the base class, with `message`.
    pub(crate) fn other(message: impl Into<String>) -> Self {
        FragmentColorError::Other {
            message: message.into(),
        }
    }

    /// An `InvalidInput` error, for a value the call can't take, with `message`.
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        FragmentColorError::InvalidInput {
            message: message.into(),
        }
    }
}

/// The fields of a shape, taken by name. Each name is one `Classify::list_fields`
/// gives, so Swift and Kotlin read the same fields Python and JavaScript set.
struct Fields(Vec<(&'static str, Field)>);

impl Fields {
    fn take(&mut self, name: &str) -> Option<Field> {
        let index = self.0.iter().position(|(field, _)| *field == name)?;
        Some(self.0.swap_remove(index).1)
    }

    fn text(&mut self, name: &str) -> String {
        match self.take(name) {
            Some(Field::Text(text)) => text,
            _ => String::new(),
        }
    }

    fn number(&mut self, name: &str) -> u64 {
        match self.take(name) {
            Some(Field::Number(number)) => number,
            _ => 0,
        }
    }

    fn kind(&mut self, name: &str) -> ErrorKind {
        match self.take(name) {
            Some(Field::Kind(kind)) => kind,
            _ => ErrorKind::Internal,
        }
    }
}

impl From<ErrorShape> for FragmentColorError {
    fn from(shape: ErrorShape) -> Self {
        let ErrorShape {
            class,
            message,
            fields,
        } = shape;
        let mut fields = Fields(fields);
        match class {
            ErrorClass::Base => FragmentColorError::Other { message },
            ErrorClass::InvalidInput => FragmentColorError::InvalidInput { message },
            ErrorClass::UnsupportedMipmapFormat => {
                FragmentColorError::UnsupportedMipmapFormat { message }
            }
            ErrorClass::MalformedImage => FragmentColorError::MalformedImage { message },
            ErrorClass::OccludedSurface => FragmentColorError::OccludedSurface { message },
            ErrorClass::Device => FragmentColorError::Device {
                message,
                kind: fields.kind("kind"),
                label: fields.text("label"),
                detail: fields.text("detail"),
            },
            ErrorClass::Limit => FragmentColorError::Limit {
                message,
                item: fields.text("item"),
                limit: fields.text("limit"),
                value: fields.number("value"),
                max: fields.number("max"),
            },
            ErrorClass::Feature => FragmentColorError::Feature {
                message,
                item: fields.text("item"),
                feature: fields.text("feature"),
            },
            ErrorClass::Binding => FragmentColorError::Binding {
                message,
                uniform: fields.text("uniform"),
                expected: fields.text("expected"),
                found: fields.text("found"),
            },
            ErrorClass::Entry => FragmentColorError::Entry {
                message,
                stage: fields.text("stage"),
            },
            ErrorClass::SceneLoad => FragmentColorError::SceneLoad { message },
            ErrorClass::Light => FragmentColorError::Light { message },
            ErrorClass::Camera => FragmentColorError::Camera { message },
            ErrorClass::Material => FragmentColorError::Material { message },
        }
    }
}

/// Each error type that Swift and Kotlin throw as the case of its class, with its fields.
macro_rules! throw_by_class {
    ($($error:ty),* $(,)?) => {$(
        impl From<$error> for FragmentColorError {
            fn from(error: $error) -> Self {
                error.shape().into()
            }
        }
    )*};
}

throw_by_class!(
    InitializationError,
    RendererError,
    ShaderError,
    TextureError,
    MeshError,
    ColorError,
    SizeError,
    ScreenRegionError,
    PassError,
    MaterialError,
    SceneLoadError,
    CameraError,
    LightError,
    ModelError,
    crate::bindings::numbers::VectorError,
);
