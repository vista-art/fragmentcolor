use lsp_doc::lsp_doc;
use thiserror::Error;

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
    #[error("'{name}' takes {expected}, not {found}")]
    BindingError {
        name: String,
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

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(#[from] wgpu::RequestAdapterError),
    #[error("Adapter not set after context()")]
    AdapterNotSet,
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create surface")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),
    /// The target's size doesn't fit the device, the error `resize` returns too.
    #[error(transparent)]
    TextureError(#[from] crate::texture::TextureError),
    #[error("Initialization error: {0}")]
    Error(String),
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
            RendererError::PassError(pass) => pass.raise(message),
            RendererError::SurfaceError(crate::target::SurfaceError::Occluded) => {
                crate::error::OccludedSurfaceError::new_err(message)
            }
            RendererError::DeviceError {
                kind,
                label,
                detail,
            } => crate::error::DeviceError::raise(kind, label, detail, message),
            RendererError::LimitError {
                item,
                limit,
                value,
                max,
            } => crate::error::raise::<crate::error::LimitError>(message, |error| {
                error.setattr("item", item)?;
                error.setattr("limit", limit)?;
                error.setattr("value", value)?;
                error.setattr("max", max)
            }),
            RendererError::FeatureError { item, feature } => {
                crate::error::raise::<crate::error::FeatureError>(message, |error| {
                    error.setattr("item", item)?;
                    error.setattr("feature", feature)
                })
            }
            RendererError::BindingError {
                name,
                expected,
                found,
            } => crate::error::raise::<crate::error::BindingError>(message, |error| {
                error.setattr("name", name)?;
                error.setattr("expected", expected)?;
                error.setattr("found", found)
            }),
            RendererError::EntryError { stage } => {
                crate::error::raise::<crate::error::EntryError>(message, |error| {
                    error.setattr("stage", stage)
                })
            }
            _ => crate::PyFragmentColorError::new_err(message),
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
            other => crate::PyFragmentColorError::new_err(other.to_string()),
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

#[cfg(wasm)]
impl RendererError {
    /// The JS `Error` this error throws. A device error also carries its `kind`, `label` and `detail`.
    pub(crate) fn into_js(self) -> JsError {
        let message = self.to_string();
        match self {
            RendererError::DeviceError {
                kind,
                label,
                detail,
            } => throw_device(kind, &label, &detail, &message),
            RendererError::LimitError {
                item,
                limit,
                value,
                max,
            } => throw_fields(
                &message,
                &[
                    ("item", item.into()),
                    ("limit", limit.into()),
                    ("value", JsValue::from_f64(value as f64)),
                    ("max", JsValue::from_f64(max as f64)),
                ],
            ),
            RendererError::FeatureError { item, feature } => throw_fields(
                &message,
                &[("item", item.into()), ("feature", feature.into())],
            ),
            RendererError::BindingError {
                name,
                expected,
                found,
            } => throw_fields(
                &message,
                &[
                    ("name", name.into()),
                    ("expected", expected.into()),
                    ("found", found.into()),
                ],
            ),
            RendererError::EntryError { stage } => {
                throw_fields(&message, &[("stage", stage.into())])
            }
            _ => JsError::new(&message),
        }
    }
}

#[cfg(wasm)]
impl From<RendererError> for wasm_bindgen::JsValue {
    fn from(error: RendererError) -> Self {
        let error: JsError = error.into_js();
        error.into()
    }
}

/// A JS `Error` with the full `message` that carries each of `fields` as a property.
#[cfg(wasm)]
fn throw_fields(message: &str, fields: &[(&str, JsValue)]) -> JsError {
    let error = JsError::new(message);
    // A clone holds the same JS object, so the properties land on the error itself.
    let value = JsValue::from(error.clone());
    for (name, field) in fields {
        // Setting a property on a new Error object never throws.
        let _ = js_sys::Reflect::set(&value, &JsValue::from_str(name), field);
    }
    error
}

/// A JS `Error` with the full `message` that carries the GPU error's `kind`,
/// the `label` of its step and the device's own `detail`.
#[cfg(wasm)]
pub(crate) fn throw_device(kind: ErrorKind, label: &str, detail: &str, message: &str) -> JsError {
    let error = JsError::new(message);
    // A clone holds the same JS object, so the properties land on the error itself.
    let value = JsValue::from(error.clone());
    // Setting a property on a new Error object never throws.
    let _ = js_sys::Reflect::set(&value, &"kind".into(), &JsValue::from(kind as u32));
    let _ = js_sys::Reflect::set(&value, &"label".into(), &JsValue::from(label));
    let _ = js_sys::Reflect::set(&value, &"detail".into(), &JsValue::from(detail));
    error
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
        js_sys::Error::new(&error.to_string()).into()
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

    // Story: SurfaceError variants are wrapped with a helpful message in Display.
    #[test]
    fn surface_error_display_includes_inner() {
        let e = RendererError::SurfaceError(crate::target::SurfaceError::OutOfMemory);
        let s = e.to_string();
        assert!(s.contains("Surface error"));
        assert!(s.contains("failed to acquire frame"));
    }
}
