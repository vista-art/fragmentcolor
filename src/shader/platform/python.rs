#![cfg(python)]

use crate::shader::lsp_doc;
use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PySequence};

#[pymethods]
impl Shader {
    #[new]
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new_py(input: &Bound<PyAny>) -> Result<Self, ShaderError> {
        if let Ok(s) = input.extract::<String>() {
            return Shader::new(s);
        }
        if let Ok(v) = input.extract::<Vec<String>>() {
            return Shader::new(v);
        }
        Err(ShaderError::ParseError(
            "Shader(input): expected str or list[str]".into(),
        ))
    }

    /// Async-shaped constructor: fetches each part of `input` from the network
    /// (URL or slug) or reads file paths, then compiles the merged source.
    /// Blocks the Python thread synchronously via `pollster::block_on` —
    /// call from a thread-pool worker if you want non-blocking behaviour.
    #[staticmethod]
    #[pyo3(name = "fetch")]
    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub fn fetch_py(input: &Bound<PyAny>) -> Result<Self, ShaderError> {
        if let Ok(s) = input.extract::<String>() {
            return pollster::block_on(Shader::fetch(s));
        }
        if let Ok(v) = input.extract::<Vec<String>>() {
            return pollster::block_on(Shader::fetch(v));
        }
        Err(ShaderError::ParseError(
            "Shader.fetch(input): expected str or list[str]".into(),
        ))
    }

    #[staticmethod]
    #[pyo3(name = "set_registry")]
    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry_py(base_url: &str) {
        Shader::set_registry(base_url);
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // If it's a Texture object, map to UniformData::Texture with id only
            if let Ok(tex) = value.bind(py).cast::<crate::texture::Texture>() {
                let meta = crate::texture::TextureMeta::with_id_only(tex.borrow().id);
                return self
                    .object
                    .set(key, UniformData::Texture(meta))
                    .map_err(|e| e.into());
            }
            // Look up the expected UniformData variant from the shader's current storage.
            // This lets us coerce a plain Python list/scalar to the exact right variant
            // (Vec2/Vec3/Vec4, IVec*, UVec*, Mat*) without relying on declaration order in
            // the FromPyObject derive, which would always match the first Vec-like variant.
            let expected = self.object.get_uniform_data(key).ok();
            let ud = py_value_to_uniform_data(value.bind(py), expected.as_ref())?;
            self.object.set(key, ud).map_err(|e| e.into())
        })
    }

    #[pyo3(name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_py(&self, key: &str) -> Result<Py<PyAny>, PyErr> {
        Python::attach(|py| -> Result<Py<PyAny>, PyErr> {
            let data = self.object.get_uniform_data(key)?;

            let object = data.into_pyobject(py)?;

            Ok(object.unbind())
        })
    }

    #[pyo3(name = "list_uniforms")]
    #[lsp_doc("docs/api/core/shader/list_uniforms.md")]
    pub fn list_uniforms_py(&self) -> Vec<String> {
        self.list_uniforms()
    }

    #[pyo3(name = "list_keys")]
    #[lsp_doc("docs/api/core/shader/list_keys.md")]
    pub fn list_keys_py(&self) -> Vec<String> {
        self.list_keys()
    }

    #[pyo3(name = "add_mesh")]
    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh_py(&self, mesh: &crate::mesh::Mesh) -> Result<(), PyErr> {
        self.add_mesh(mesh).map_err(|e| e.into())
    }

    #[pyo3(name = "remove_mesh")]
    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh_py(&self, mesh: &crate::mesh::Mesh) {
        self.remove_mesh(mesh)
    }

    #[pyo3(name = "remove_meshes")]
    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes_py(&self, list: Vec<crate::mesh::Mesh>) {
        for m in list.iter() {
            self.remove_mesh(m);
        }
    }

    #[pyo3(name = "clear_meshes")]
    #[lsp_doc("docs/api/core/shader/clear_meshes.md")]
    pub fn clear_meshes_py(&self) {
        self.clear_meshes()
    }

    #[pyo3(name = "validate_mesh")]
    #[lsp_doc("docs/api/core/shader/validate_mesh.md")]
    pub fn validate_mesh_py(&self, mesh: &crate::mesh::Mesh) -> Result<(), PyErr> {
        self.validate_mesh(mesh).map_err(|e| e.into())
    }

    #[pyo3(name = "is_compute")]
    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute_py(&self) -> bool {
        self.is_compute()
    }

    // Internal duck-typed interface used by PyRenderable dispatch — not part of public docs.
    #[doc(hidden)]
    pub fn renderable_type(&self) -> &'static str {
        "Shader"
    }

    // Internal duck-typed interface used by PyRenderable dispatch — not part of public docs.
    #[doc(hidden)]
    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.pass.clone()])
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    #[lsp_doc("docs/api/core/shader/default.md")]
    pub fn default_py() -> Self {
        Shader::default()
    }

    #[staticmethod]
    #[pyo3(name = "from_vertex")]
    #[lsp_doc("docs/api/core/shader/from_vertex.md")]
    pub fn from_vertex_py(v: &crate::mesh::Vertex) -> Self {
        Shader::from_vertex(v)
    }

    #[staticmethod]
    #[pyo3(name = "from_mesh")]
    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh_py(mesh: &crate::mesh::Mesh) -> Self {
        Shader::from_mesh(mesh)
    }
}

/// Convert a Python value to `UniformData`, guided by the expected variant when
/// the caller passes a plain list or scalar.  Without guidance the `FromPyObject`
/// derive would always pick the first matching Vec-like variant (e.g. `Vec2`) for
/// any Python list, causing a discriminant mismatch inside `storage.update`.
///
/// Resolution order:
/// 1. If `expected` is Some, use its discriminant to coerce the Python value.
/// 2. Fall back to the `FromPyObject` derive (handles all other cases).
fn py_value_to_uniform_data(
    value: &Bound<'_, PyAny>,
    expected: Option<&UniformData>,
) -> PyResult<UniformData> {
    // If the caller passes a sequence (list/tuple) and we know the target variant, coerce.
    if let Some(expected) = expected {
        // Try to read as a sequence of floats
        if let Ok(seq) = value.cast::<PySequence>() {
            let len = seq.len().unwrap_or(0);
            match expected {
                UniformData::Vec2(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Vec2(v));
                }
                UniformData::Vec3(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Vec3(v));
                }
                UniformData::Vec4(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Vec4(v));
                }
                UniformData::IVec2(_) => {
                    let v: Vec<i32> = seq.extract()?;
                    return Ok(UniformData::IVec2(v));
                }
                UniformData::IVec3(_) => {
                    let v: Vec<i32> = seq.extract()?;
                    return Ok(UniformData::IVec3(v));
                }
                UniformData::IVec4(_) => {
                    let v: Vec<i32> = seq.extract()?;
                    return Ok(UniformData::IVec4(v));
                }
                UniformData::UVec2(_) => {
                    let v: Vec<u32> = seq.extract()?;
                    return Ok(UniformData::UVec2(v));
                }
                UniformData::UVec3(_) => {
                    let v: Vec<u32> = seq.extract()?;
                    return Ok(UniformData::UVec3(v));
                }
                UniformData::UVec4(_) => {
                    let v: Vec<u32> = seq.extract()?;
                    return Ok(UniformData::UVec4(v));
                }
                UniformData::Mat2(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Mat2(v));
                }
                UniformData::Mat3(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Mat3(v));
                }
                UniformData::Mat4(_) => {
                    let v: Vec<f32> = seq.extract()?;
                    return Ok(UniformData::Mat4(v));
                }
                // For unknown expected types, fall through to length-based inference below.
                _ => {}
            }

            // No `expected` match: infer from sequence length.
            // Try f32 first, then i32, then u32.
            if let Ok(v) = seq.extract::<Vec<f32>>() {
                return Ok(match len {
                    2 => UniformData::Vec2(v),
                    3 => UniformData::Vec3(v),
                    4 => UniformData::Vec4(v),
                    9 => UniformData::Mat3(v),
                    16 => UniformData::Mat4(v),
                    _ => UniformData::Vec2(v),
                });
            }
            if let Ok(v) = seq.extract::<Vec<i32>>() {
                return Ok(match len {
                    2 => UniformData::IVec2(v),
                    3 => UniformData::IVec3(v),
                    4 => UniformData::IVec4(v),
                    _ => UniformData::IVec2(v),
                });
            }
            if let Ok(v) = seq.extract::<Vec<u32>>() {
                return Ok(match len {
                    2 => UniformData::UVec2(v),
                    3 => UniformData::UVec3(v),
                    4 => UniformData::UVec4(v),
                    _ => UniformData::UVec2(v),
                });
            }
        }

        // Scalar coercions guided by expected type
        match expected {
            UniformData::Float(_) => {
                if let Ok(v) = value.extract::<f32>() {
                    return Ok(UniformData::Float(v));
                }
            }
            UniformData::Int(_) => {
                if let Ok(v) = value.extract::<i32>() {
                    return Ok(UniformData::Int(v));
                }
            }
            UniformData::UInt(_) => {
                if let Ok(v) = value.extract::<u32>() {
                    return Ok(UniformData::UInt(v));
                }
            }
            UniformData::Bool(_) => {
                if let Ok(v) = value.extract::<bool>() {
                    return Ok(UniformData::Bool(v));
                }
            }
            _ => {}
        }
    } else {
        // No expected type at all: try length-based inference for sequences.
        if let Ok(seq) = value.cast::<PySequence>() {
            let len = seq.len().unwrap_or(0);
            if let Ok(v) = seq.extract::<Vec<f32>>() {
                return Ok(match len {
                    2 => UniformData::Vec2(v),
                    3 => UniformData::Vec3(v),
                    4 => UniformData::Vec4(v),
                    9 => UniformData::Mat3(v),
                    16 => UniformData::Mat4(v),
                    _ => UniformData::Vec2(v),
                });
            }
            if let Ok(v) = seq.extract::<Vec<i32>>() {
                return Ok(match len {
                    2 => UniformData::IVec2(v),
                    3 => UniformData::IVec3(v),
                    4 => UniformData::IVec4(v),
                    _ => UniformData::IVec2(v),
                });
            }
            if let Ok(v) = seq.extract::<Vec<u32>>() {
                return Ok(match len {
                    2 => UniformData::UVec2(v),
                    3 => UniformData::UVec3(v),
                    4 => UniformData::UVec4(v),
                    _ => UniformData::UVec2(v),
                });
            }
        }
    }

    // Final fallback: let the FromPyObject derive do its best.
    value.extract::<UniformData>()
}
