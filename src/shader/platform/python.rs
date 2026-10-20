#![cfg(python)]

use crate::shader::{UniformInput, lsp_doc};
use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyBool, PyByteArray, PyBytes, PyMemoryView};

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
    pub fn set_py(&self, key: &str, value: &Bound<'_, PyAny>) -> Result<(), PyErr> {
        Ok(self.object.set_input(key, uniform_input(key, value)?)?)
    }

    #[pyo3(name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_py(&self, key: &str) -> Result<UniformData, ShaderError> {
        self.object.get_uniform_data(key)
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

/// Sorts a Python value into a loose uniform input. The key's declared WGSL type does the rest.
/// `bytes`, `bytearray` and `memoryview` are raw bytes, and any other sequence is numbers.
fn uniform_input(key: &str, value: &Bound<'_, PyAny>) -> PyResult<UniformInput> {
    if let Ok(texture) = value.cast::<crate::texture::Texture>() {
        return Ok(UniformInput::Texture(texture.borrow().id));
    }
    if let Ok(flag) = value.cast::<PyBool>() {
        return Ok(UniformInput::Bool(flag.is_true()));
    }
    if value.is_instance_of::<PyBytes>() || value.is_instance_of::<PyByteArray>() {
        return Ok(UniformInput::Bytes(value.extract()?));
    }
    if value.is_instance_of::<PyMemoryView>() {
        return Ok(UniformInput::Bytes(
            value.call_method0("tobytes")?.extract()?,
        ));
    }
    // Vec extraction follows the sequence protocol, so NumPy arrays count too.
    if let Ok(numbers) = value.extract::<Vec<f64>>() {
        return Ok(UniformInput::Numbers(numbers));
    }
    if let Ok(number) = value.extract::<f64>() {
        return Ok(UniformInput::Number(number));
    }
    let kind = value.get_type().name()?;
    Err(ShaderError::TypeMismatch(format!(
        "{key}: expected a number, a sequence of numbers, bytes, or a Texture, got {kind}"
    ))
    .into())
}
