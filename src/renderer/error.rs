use lsp_doc::lsp_doc;
use thiserror::Error;

use crate::shader::error::describe;

#[cfg(any(python, wasm, test))]
use crate::error::{Classify, ErrorClass};
#[cfg(python)]
use crate::error::{raise_class, raise_fields};
#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[lsp_doc("docs/api/core/error_kind/error_kind.md")]
pub enum ErrorKind {
    /// The GPU refused a call that breaks a rule of the API.
    Validation,
    /// The GPU failed for a reason of its own, such as a driver bug.
    Internal,
    /// The device ran out of memory.
    Memory,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ErrorKind::Validation => "validation",
            ErrorKind::Internal => "internal",
            ErrorKind::Memory => "memory",
        };
        formatter.write_str(name)
    }
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Context not initialized")]
    NoContext,
    #[error("Surface error: failed to acquire frame: {0}")]
    SurfaceError(#[from] crate::target::SurfaceError),
    #[error("Failed to create texture: {0}")]
    CreateTextureError(String),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::shader::ShaderError),
    #[error("Mesh error: {0}")]
    MeshError(#[from] crate::mesh::MeshError),
    #[error("Pass error: {0}")]
    PassError(#[from] crate::pass::PassError),
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    /// A GPU error an error scope caught, labeled with the step that raised it.
    #[error("Device error ({kind}) in '{label}': {detail}")]
    DeviceError {
        kind: ErrorKind,
        label: String,
        detail: String,
    },
    #[error("Texture error: {0}")]
    TextureError(#[from] crate::texture::TextureError),
    #[error("Texture {0} not found")]
    TextureNotFoundError(crate::texture::TextureId),
    #[error(
        "Storage binding '{0}' not found — either the shader doesn't declare it or no render pass has uploaded it yet"
    )]
    StorageBindingNotFound(String),
    #[error("Depth sample_count mismatch: depth={depth} pass={pass}")]
    DepthSampleCountMismatch { depth: u32, pass: u32 },
    /// A depth target whose width and height differ from the color attachment's.
    #[error(
        "Depth size mismatch: depth={}x{} pass={}x{}",
        .depth[0], .depth[1], .pass[0], .pass[1]
    )]
    DepthSizeMismatch { depth: [u32; 2], pass: [u32; 2] },
    /// Something in a shader or mesh past one of the device's limits, such as a bind
    /// group index past `max_bind_groups` or a vertex stride past `max_vertex_buffer_array_stride`.
    #[error("{item} needs {limit} of at least {value}, and the device has {max}")]
    LimitError {
        item: String,
        limit: String,
        value: u64,
        max: u64,
    },
    /// Something in a shader that needs a feature or capability the device lacks.
    #[error("{item} needs {feature}, which the device lacks")]
    FeatureError { item: String, feature: String },
    /// A resource bound to a shader slot that can't take it, or a slot with nothing bound.
    #[error("'{uniform}' takes {expected}, not {found}")]
    BindingError {
        uniform: String,
        expected: String,
        found: String,
    },
    /// A shader without the entry point its pass runs.
    #[error("the shader has no {stage} entry point, which its pass runs")]
    EntryError { stage: String },
    /// A target that another Renderer created, whose frames work on another device.
    #[error("The target belongs to another Renderer; render it with the Renderer that created it")]
    ForeignTarget,
    #[error("initialization error: {0}")]
    InitializationError(#[from] InitializationError),
    #[cfg(not(wasm))]
    #[error("Network request error: {0}")]
    NetworkRequestError(#[from] crate::net::NetworkError),
    #[error("Malformed input error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Renderer error: {0}")]
    Error(String),
}

#[cfg(any(python, wasm, mobile, test))]
impl RendererError {
    /// Whether the frame failed because the window is hidden or minimized, which every
    /// binding raises as its own class or case, so a render loop can skip the frame.
    pub(crate) fn is_occluded(&self) -> bool {
        matches!(
            self,
            RendererError::SurfaceError(crate::target::SurfaceError::Occluded)
        )
    }
}

/// A property a typed refusal carries on the error a binding raises.
#[cfg(any(python, wasm, test))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Field {
    Text(String),
    Number(u64),
}

#[cfg(any(python, wasm, test))]
impl RendererError {
    /// The properties a typed refusal sets, by name, on the error Python
    /// and JavaScript raise. Any other error lists none here, and `DeviceError`
    /// sets its `kind`, `label` and `detail` in `DeviceError::raise` and `throw_device`.
    pub(crate) fn list_fields(&self) -> Vec<(&'static str, Field)> {
        let text = |value: &String| Field::Text(value.clone());
        match self {
            RendererError::LimitError {
                item,
                limit,
                value,
                max,
            } => vec![
                ("item", text(item)),
                ("limit", text(limit)),
                ("value", Field::Number(*value)),
                ("max", Field::Number(*max)),
            ],
            RendererError::FeatureError { item, feature } => {
                vec![("item", text(item)), ("feature", text(feature))]
            }
            RendererError::BindingError {
                uniform,
                expected,
                found,
            } => vec![
                ("uniform", text(uniform)),
                ("expected", text(expected)),
                ("found", text(found)),
            ],
            RendererError::EntryError { stage } => vec![("stage", text(stage))],
            _ => Vec::new(),
        }
    }
}

#[cfg(any(python, wasm, test))]
impl Classify for RendererError {
    fn classify(&self) -> ErrorClass {
        match self {
            error if error.is_occluded() => ErrorClass::OccludedSurface,
            RendererError::TextureError(error) => error.classify(),
            RendererError::InitializationError(error) => error.classify(),
            RendererError::PassError(error) => error.classify(),
            RendererError::MeshError(error) => error.classify(),
            RendererError::ShaderError(error) => error.classify(),
            RendererError::DeviceError { .. } => ErrorClass::Device,
            RendererError::LimitError { .. } => ErrorClass::Limit,
            RendererError::FeatureError { .. } => ErrorClass::Feature,
            RendererError::BindingError { .. } => ErrorClass::Binding,
            RendererError::EntryError { .. } => ErrorClass::Entry,
            _ => ErrorClass::Base,
        }
    }
}

#[derive(Error, Debug)]
pub enum InitializationError {
    /// No GPU adapter fits, with wgpu's reason.
    #[error("Failed to find a compatible GPU adapter: {0}")]
    AdapterError(String),
    #[error("Adapter not set after context()")]
    AdapterNotSet,
    /// The adapter refused to open a device, with wgpu's reason.
    #[error("Failed to create device: {0}")]
    DeviceError(String),
    /// The window or canvas gave no surface, with wgpu's reason.
    #[error("Failed to create surface: {0}")]
    SurfaceError(String),
    /// The target's size doesn't fit the device, the error `resize` returns too.
    #[error(transparent)]
    TextureError(#[from] crate::texture::TextureError),
    #[error("Initialization error: {0}")]
    Error(String),
}

/// A failure of wgpu while it opens the GPU, which `InitializationError` carries as text.
#[derive(Debug, Error)]
pub(crate) enum WgpuFailure {
    #[error(transparent)]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error(transparent)]
    Device(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    Surface(#[from] wgpu::CreateSurfaceError),
}

impl From<WgpuFailure> for InitializationError {
    fn from(failure: WgpuFailure) -> Self {
        match failure {
            WgpuFailure::Adapter(error) => Self::AdapterError(describe(&error)),
            WgpuFailure::Device(error) => Self::DeviceError(describe(&error)),
            WgpuFailure::Surface(error) => Self::SurfaceError(describe(&error)),
        }
    }
}

#[cfg(any(python, wasm, test))]
impl Classify for InitializationError {
    fn classify(&self) -> ErrorClass {
        match self {
            InitializationError::TextureError(error) => error.classify(),
            _ => ErrorClass::Base,
        }
    }
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for RendererError {
    fn from(e: pyo3::PyErr) -> Self {
        RendererError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<RendererError> for pyo3::PyErr {
    fn from(e: RendererError) -> Self {
        let message = e.to_string();
        match e {
            RendererError::TextureError(texture) => texture.into(),
            RendererError::InitializationError(init) => init.into(),
            RendererError::DeviceError {
                kind,
                label,
                detail,
            } => crate::error::DeviceError::raise(kind, label, detail, message),
            other => raise_fields(other.classify(), message, other.list_fields()),
        }
    }
}

#[cfg(python)]
impl From<pyo3::PyErr> for InitializationError {
    fn from(e: pyo3::PyErr) -> Self {
        InitializationError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<InitializationError> for pyo3::PyErr {
    fn from(e: InitializationError) -> Self {
        match e {
            InitializationError::TextureError(texture) => texture.into(),
            other => raise_class(other.classify(), other.to_string()),
        }
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for RendererError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        RendererError::Error(error_string)
    }
}

/// The JS `Error` a renderer error throws, named after its class, with the fields of a
/// typed refusal, or the `kind`, `label` and `detail` of a device error.
#[cfg(wasm)]
impl From<RendererError> for crate::error::WebError {
    fn from(error: RendererError) -> Self {
        let message = error.to_string();
        match error {
            RendererError::DeviceError {
                kind,
                label,
                detail,
            } => throw_device(kind, &label, &detail, &message),
            other => Self::with_fields(other.classify(), &message, &other.list_fields()),
        }
    }
}

#[cfg(wasm)]
impl From<RendererError> for wasm_bindgen::JsValue {
    fn from(error: RendererError) -> Self {
        crate::error::WebError::from(error).into()
    }
}

/// A `DeviceError` with the full `message` that carries the GPU error's `kind`,
/// the `label` of its step and the device's own `detail`.
#[cfg(wasm)]
pub(crate) fn throw_device(
    kind: ErrorKind,
    label: &str,
    detail: &str,
    message: &str,
) -> crate::error::WebError {
    let fields = [
        ("kind", Field::Number(kind as u64)),
        ("label", Field::Text(label.into())),
        ("detail", Field::Text(detail.into())),
    ];
    crate::error::WebError::with_fields(ErrorClass::Device, message, &fields)
}

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for InitializationError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        InitializationError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<InitializationError> for wasm_bindgen::JsValue {
    fn from(error: InitializationError) -> Self {
        crate::error::WebError::from(error).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: RendererError variants format messages and From conversions work.
    #[test]
    fn renderer_error_variants_and_from() {
        // NoContext
        let e: RendererError = RendererError::NoContext;
        assert!(e.to_string().contains("Context not initialized"));

        // InitializationError conversion
        let ie = InitializationError::Error("boom".into());
        let e2: RendererError = ie.into();
        assert!(matches!(e2, RendererError::InitializationError(_)));

        // IoError conversion
        let io = std::io::Error::other("oops");
        let e3: RendererError = io.into();
        assert!(matches!(e3, RendererError::IoError(_)));

        // NetworkRequestError (non-WASM)
        #[cfg(not(wasm))]
        {
            let net = crate::net::NetworkError("connection refused".into());
            let e4: RendererError = net.into();
            assert!(matches!(e4, RendererError::NetworkRequestError(_)));
        }
    }

    // Story: a device error names its kind, the step that raised it and the GPU's message.
    #[test]
    fn device_error_names_kind_and_label() {
        // Arrange
        let error = RendererError::DeviceError {
            kind: ErrorKind::Memory,
            label: "frame".into(),
            detail: "Out of Memory".into(),
        };

        // Act
        let text = error.to_string();

        // Assert
        assert_eq!(text, "Device error (memory) in 'frame': Out of Memory");
    }

    // Story: each typed refusal carries its own fields, and none of them shadows
    // the `name` or `message` that Python and JavaScript errors already have.
    #[test]
    fn refusals_carry_their_fields() {
        // Arrange
        let text = |value: &str| Field::Text(value.into());
        let limit = RendererError::LimitError {
            item: "group 9".into(),
            limit: "max_bind_groups".into(),
            value: 10,
            max: 4,
        };
        let feature = RendererError::FeatureError {
            item: "a bgra8unorm storage texture".into(),
            feature: "BGRA8UNORM_STORAGE".into(),
        };
        let binding = RendererError::BindingError {
            uniform: "image".into(),
            expected: "a texture".into(),
            found: "none, since it was never set".into(),
        };
        let entry = RendererError::EntryError {
            stage: "fragment".into(),
        };

        // Act
        let fields = [&limit, &feature, &binding, &entry].map(|error| error.list_fields());

        // Assert
        let [limit, feature, binding, entry] = fields;
        let expected = vec![
            ("item", text("group 9")),
            ("limit", text("max_bind_groups")),
            ("value", Field::Number(10)),
            ("max", Field::Number(4)),
        ];
        assert_eq!(limit, expected);
        let expected = vec![
            ("item", text("a bgra8unorm storage texture")),
            ("feature", text("BGRA8UNORM_STORAGE")),
        ];
        assert_eq!(feature, expected);
        let expected = vec![
            ("uniform", text("image")),
            ("expected", text("a texture")),
            ("found", text("none, since it was never set")),
        ];
        assert_eq!(binding, expected);
        assert_eq!(entry, vec![("stage", text("fragment"))]);
        assert!(RendererError::NoContext.list_fields().is_empty());
    }

    // Story: SurfaceError variants are wrapped with a helpful message in Display.
    #[test]
    fn surface_error_display_includes_inner() {
        let e = RendererError::SurfaceError(crate::target::SurfaceError::OutOfMemory);
        let s = e.to_string();
        assert!(s.contains("Surface error"));
        assert!(s.contains("failed to acquire frame"));
    }

    // Story: only a hidden window's frame counts as occluded, the one decision Python,
    // JavaScript, Swift and Kotlin read to raise their occluded class or case.
    #[test]
    fn only_a_hidden_window_is_occluded() {
        use crate::target::SurfaceError;
        let hidden = RendererError::SurfaceError(SurfaceError::Occluded);
        assert!(hidden.is_occluded());
        assert_eq!(
            hidden.to_string(),
            "Surface error: failed to acquire frame: surface is occluded"
        );
        for other in [
            RendererError::SurfaceError(SurfaceError::Lost),
            RendererError::SurfaceError(SurfaceError::OutOfMemory),
            RendererError::NoContext,
        ] {
            assert!(!other.is_occluded(), "{other}");
        }
    }
}
