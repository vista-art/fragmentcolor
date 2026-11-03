//! The uniffi `FragmentColorError` that Swift and Kotlin throw.

use crate::scene::SceneLoadError;
use crate::texture::TextureError;
use crate::{ColorError, ErrorKind, InitializationError, MeshError, PassError};
use crate::{RendererError, ShaderError, SurfaceError};

/// Mobile-facing error type, one typed error for Swift and Kotlin to match on.
///
/// Most internal errors flatten to their `Display` text. A GPU error keeps its kind,
/// its step's label and the device's own detail apart, a shader the device refuses
/// keeps the fields of its refusal, and a hidden window's frame is `Occluded`, so a
/// render loop can skip it. The exported `Display` gives Swift's `description` and
/// `localizedDescription`, and Kotlin's `toString()`, the full text of every case.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi::export(Display)]
pub enum FragmentColorError {
    #[error("{0}")]
    Init(String),
    #[error("{0}")]
    Render(String),
    #[error("{0}")]
    Shader(String),
    #[error("Device error ({kind}) in '{label}': {detail}")]
    Device {
        kind: ErrorKind,
        label: String,
        detail: String,
    },
    #[error("{item} needs {limit} of at least {value}, and the device has {max}")]
    Limit {
        item: String,
        limit: String,
        value: u64,
        max: u64,
    },
    #[error("{item} needs {feature}, which the device lacks")]
    Feature { item: String, feature: String },
    #[error("'{uniform}' takes {expected}, not {found}")]
    Binding {
        uniform: String,
        expected: String,
        found: String,
    },
    #[error("the shader has no {stage} entry point, which its pass runs")]
    Entry { stage: String },
    #[error("{}", RendererError::SurfaceError(SurfaceError::Occluded))]
    Occluded,
}

impl FragmentColorError {
    /// The case a renderer error that isn't a GPU error takes.
    fn from_refusal(error: RendererError) -> Self {
        use RendererError as Refusal;
        match error {
            Refusal::LimitError {
                item,
                limit,
                value,
                max,
            } => FragmentColorError::Limit {
                item,
                limit,
                value,
                max,
            },
            Refusal::FeatureError { item, feature } => {
                FragmentColorError::Feature { item, feature }
            }
            Refusal::BindingError {
                uniform,
                expected,
                found,
            } => FragmentColorError::Binding {
                uniform,
                expected,
                found,
            },
            Refusal::EntryError { stage } => FragmentColorError::Entry { stage },
            other => FragmentColorError::Render(other.to_string()),
        }
    }
}

impl From<InitializationError> for FragmentColorError {
    fn from(error: InitializationError) -> Self {
        match error {
            InitializationError::TextureError(texture) => texture.into(),
            other => FragmentColorError::Init(other.to_string()),
        }
    }
}

impl From<RendererError> for FragmentColorError {
    fn from(error: RendererError) -> Self {
        match error {
            error if error.is_occluded() => FragmentColorError::Occluded,
            RendererError::InitializationError(error) => error.into(),
            RendererError::DeviceError {
                kind,
                label,
                detail,
            } => FragmentColorError::Device {
                kind,
                label,
                detail,
            },
            error => FragmentColorError::from_refusal(error),
        }
    }
}

impl From<ShaderError> for FragmentColorError {
    fn from(error: ShaderError) -> Self {
        FragmentColorError::Shader(error.to_string())
    }
}

impl From<TextureError> for FragmentColorError {
    fn from(error: TextureError) -> Self {
        match error {
            TextureError::DeviceError {
                kind,
                label,
                detail,
            } => FragmentColorError::Device {
                kind,
                label,
                detail,
            },
            error => FragmentColorError::Render(error.to_string()),
        }
    }
}

impl From<MeshError> for FragmentColorError {
    fn from(error: MeshError) -> Self {
        FragmentColorError::Render(error.to_string())
    }
}

impl From<ColorError> for FragmentColorError {
    fn from(error: ColorError) -> Self {
        FragmentColorError::Render(error.to_string())
    }
}

impl From<PassError> for FragmentColorError {
    fn from(error: PassError) -> Self {
        FragmentColorError::Render(error.to_string())
    }
}

impl From<SceneLoadError> for FragmentColorError {
    fn from(error: SceneLoadError) -> Self {
        FragmentColorError::Render(error.to_string())
    }
}
