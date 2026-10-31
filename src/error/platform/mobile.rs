//! The uniffi `FragmentColorError` that Swift and Kotlin throw.

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
        kind: crate::ErrorKind,
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
    #[error(
        "{}",
        crate::RendererError::SurfaceError(crate::SurfaceError::Occluded)
    )]
    Occluded,
}

impl FragmentColorError {
    /// The case a renderer error that isn't a GPU error takes.
    fn from_refusal(error: crate::RendererError) -> Self {
        use crate::RendererError as Refusal;
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

impl From<crate::InitializationError> for FragmentColorError {
    fn from(e: crate::InitializationError) -> Self {
        match e {
            crate::InitializationError::TextureError(texture) => texture.into(),
            other => FragmentColorError::Init(other.to_string()),
        }
    }
}

impl From<crate::RendererError> for FragmentColorError {
    fn from(e: crate::RendererError) -> Self {
        match e {
            e if e.is_occluded() => FragmentColorError::Occluded,
            crate::RendererError::InitializationError(e) => e.into(),
            crate::RendererError::DeviceError {
                kind,
                label,
                detail,
            } => FragmentColorError::Device {
                kind,
                label,
                detail,
            },
            e => FragmentColorError::from_refusal(e),
        }
    }
}

impl From<crate::ShaderError> for FragmentColorError {
    fn from(e: crate::ShaderError) -> Self {
        FragmentColorError::Shader(e.to_string())
    }
}

impl From<crate::texture::TextureError> for FragmentColorError {
    fn from(e: crate::texture::TextureError) -> Self {
        match e {
            crate::texture::TextureError::DeviceError {
                kind,
                label,
                detail,
            } => FragmentColorError::Device {
                kind,
                label,
                detail,
            },
            e => FragmentColorError::Render(e.to_string()),
        }
    }
}

impl From<crate::MeshError> for FragmentColorError {
    fn from(e: crate::MeshError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

impl From<crate::ColorError> for FragmentColorError {
    fn from(e: crate::ColorError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

impl From<crate::PassError> for FragmentColorError {
    fn from(e: crate::PassError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}

impl From<crate::scene::SceneLoadError> for FragmentColorError {
    fn from(e: crate::scene::SceneLoadError) -> Self {
        FragmentColorError::Render(e.to_string())
    }
}
