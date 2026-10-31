#![cfg(python)]

use crate::core::shader::{UniformInput, lsp_doc};
use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyByteArray, PyBytes, PyMemoryView};

#[pymethods]
impl Shader {
    #[new]
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new_py(input: PyShaderInput) -> Result<Self, ShaderError> {
        Shader::new(input)
    }

    /// Async-shaped constructor: fetches each part of `input` from the network
    /// (URL or slug) or reads file paths, then compiles the merged source.
    /// Blocks the Python thread synchronously via `pollster::block_on` —
    /// call from a thread-pool worker if you want non-blocking behavior.
    #[staticmethod]
    #[pyo3(name = "fetch")]
    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub fn fetch_py(input: PyShaderInput) -> Result<Self, ShaderError> {
        pollster::block_on(Shader::fetch(input))
    }

    #[staticmethod]
    #[pyo3(name = "set_registry")]
    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry_py(base_url: &str) {
        Shader::set_registry(base_url);
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_py(&self, key: &str, value: PyUniformValue<'_>) -> Result<(), PyErr> {
        Ok(self.object.set_input(key, value.into_input()?)?)
    }

    #[pyo3(name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_py(&self, key: &str) -> Result<PyUniformData, ShaderError> {
        Ok(self.object.get_uniform_data(key)?.into())
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
    pub fn add_mesh_py(&self, mesh: &crate::Mesh) -> Result<(), PyErr> {
        Ok(self.add_mesh(mesh)?)
    }

    #[pyo3(name = "remove_mesh")]
    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh_py(&self, mesh: &crate::Mesh) {
        self.remove_mesh(mesh)
    }

    #[pyo3(name = "remove_meshes")]
    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes_py(&self, list: Vec<crate::Mesh>) {
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
    pub fn validate_mesh_py(&self, mesh: &crate::Mesh) -> Result<(), PyErr> {
        Ok(self.validate_mesh(mesh)?)
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
    pub fn from_vertex_py(v: &crate::Vertex) -> PyResult<Self> {
        Ok(Shader::from_vertex(v)?)
    }

    #[staticmethod]
    #[pyo3(name = "from_mesh")]
    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh_py(mesh: &crate::Mesh) -> PyResult<Self> {
        Ok(Shader::from_mesh(mesh)?)
    }
}

/// A `Shader(...)` or `Shader.fetch(...)` input from Python: one source, or a list of parts.
/// Any other type raises `TypeError`.
#[derive(FromPyObject)]
pub enum PyShaderInput {
    #[pyo3(annotation = "str")]
    Source(String),
    #[pyo3(annotation = "list[str]")]
    Parts(Vec<String>),
}

impl From<PyShaderInput> for crate::ShaderInput {
    fn from(input: PyShaderInput) -> Self {
        match input {
            PyShaderInput::Source(source) => source.into(),
            PyShaderInput::Parts(parts) => parts.into(),
        }
    }
}

/// A `Shader.set` value from Python, tried in this order.
/// The key's declared WGSL type does the rest. Vec extraction follows
/// the sequence protocol, so a NumPy array counts as a sequence of numbers,
/// and a nested list or a 2-D array as a matrix, one column per inner list.
#[derive(FromPyObject)]
pub enum PyUniformValue<'py> {
    #[pyo3(annotation = "Texture")]
    Texture(PyRef<'py, crate::texture::Texture>),
    #[pyo3(annotation = "bool")]
    Bool(bool),
    #[pyo3(annotation = "bytes")]
    Bytes(Bound<'py, PyBytes>),
    #[pyo3(annotation = "bytearray")]
    ByteArray(Bound<'py, PyByteArray>),
    #[pyo3(annotation = "memoryview")]
    MemoryView(Bound<'py, PyMemoryView>),
    #[pyo3(annotation = "Sequence[float]")]
    Numbers(Vec<f64>),
    #[pyo3(annotation = "Sequence[Sequence[float]]")]
    Matrix(Vec<Vec<f64>>),
    #[pyo3(annotation = "float")]
    Number(f64),
}

impl PyUniformValue<'_> {
    /// The loose input this value stands for.
    /// A memoryview is copied out with `tobytes`, whatever its item format.
    fn into_input(self) -> PyResult<UniformInput> {
        Ok(match self {
            Self::Texture(texture) => UniformInput::Texture(*texture.id()),
            Self::Bool(flag) => UniformInput::Bool(flag),
            Self::Bytes(bytes) => UniformInput::Bytes(bytes.as_bytes().to_vec()),
            Self::ByteArray(bytes) => UniformInput::Bytes(bytes.to_vec()),
            Self::MemoryView(view) => UniformInput::Bytes(view.call_method0("tobytes")?.extract()?),
            Self::Numbers(numbers) => UniformInput::Numbers(numbers),
            // Each inner list is one column, as in Rust's [[f32; 4]; 4].
            Self::Matrix(columns) => UniformInput::Numbers(columns.concat()),
            Self::Number(number) => UniformInput::Number(number),
        })
    }
}

/// A `Shader.get` result for Python, in the shapes a key's current bytes decode to.
/// Struct, array, and buffer roots decode to raw bytes, so they arrive as `bytes`.
#[derive(IntoPyObject)]
pub enum PyUniformData {
    Bool(bool),
    UInt(u32),
    Int(i32),
    Float(f32),
    Floats(Vec<f32>),
    UInts(Vec<u32>),
    Ints(Vec<i32>),
    Texture(crate::texture::TextureMeta),
    Sampler(crate::texture::SamplerInfo),
    Bytes(Vec<u8>),
}

impl From<UniformData> for PyUniformData {
    fn from(data: UniformData) -> Self {
        use UniformData as U;
        match data {
            U::Bool(value) => Self::Bool(value),
            U::UInt(value) => Self::UInt(value),
            U::Int(value) => Self::Int(value),
            U::Float(value) => Self::Float(value),
            U::Vec2(v) | U::Vec3(v) | U::Vec4(v) | U::Mat2(v) | U::Mat3(v) | U::Mat4(v) => {
                Self::Floats(v)
            }
            U::UVec2(v) | U::UVec3(v) | U::UVec4(v) => Self::UInts(v),
            U::IVec2(v) | U::IVec3(v) | U::IVec4(v) => Self::Ints(v),
            U::Texture(meta) => Self::Texture(meta),
            U::Sampler(info) => Self::Sampler(info),
            U::Bytes(bytes) => Self::Bytes(bytes),
            shape @ (U::Array(_) | U::Struct(_) | U::Storage(_) | U::PushConstant(_)) => {
                Self::Bytes(shape.to_bytes())
            }
        }
    }
}
