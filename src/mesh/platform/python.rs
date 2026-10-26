#![cfg(python)]

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::mesh::{Instance, Mesh, MeshError, Vertex, VertexData};
use crate::platforms::python::read_unsigned;
use lsp_doc::lsp_doc;

/// A vertex or instance attribute from Python:
/// a number, or a sequence of 1 to 4 numbers such as a NumPy array.
/// Sequences come first, so a NumPy array never passes through `float()`.
/// The numbers take the type the shader declares when the mesh attaches to it.
#[derive(FromPyObject)]
pub enum PyAttributeValue {
    #[pyo3(annotation = "Sequence[float]")]
    Numbers(Vec<f64>),
    #[pyo3(annotation = "float")]
    Number(f64),
}

impl TryFrom<PyAttributeValue> for VertexData {
    type Error = MeshError;

    fn try_from(value: PyAttributeValue) -> Result<Self, Self::Error> {
        match value {
            PyAttributeValue::Numbers(numbers) => VertexData::try_from(numbers.as_slice()),
            PyAttributeValue::Number(number) => Ok(number.into()),
        }
    }
}

/// A mesh error raises `FragmentColorError`.
impl From<MeshError> for PyErr {
    fn from(error: MeshError) -> Self {
        crate::error::PyFragmentColorError::new_err(error.to_string())
    }
}

/// A `Vertex` position from Python: a `Vertex` to copy,
/// a number, or 1 to 4 numbers, tried in that order.
#[derive(FromPyObject)]
pub enum PyVertexPosition {
    #[pyo3(annotation = "Vertex")]
    Vertex(Vertex),
    #[pyo3(annotation = "float")]
    X(f32),
    #[pyo3(annotation = "[x]")]
    Single([f32; 1]),
    #[pyo3(annotation = "[x, y]")]
    Xy([f32; 2]),
    #[pyo3(annotation = "[x, y, z]")]
    Xyz([f32; 3]),
    #[pyo3(annotation = "[x, y, z, w]")]
    Xyzw([f32; 4]),
}

impl From<PyVertexPosition> for Vertex {
    fn from(position: PyVertexPosition) -> Self {
        match position {
            PyVertexPosition::Vertex(vertex) => vertex,
            PyVertexPosition::X(x) => Vertex::new(x),
            PyVertexPosition::Single(x) => Vertex::new(x),
            PyVertexPosition::Xy(xy) => Vertex::new(xy),
            PyVertexPosition::Xyz(xyz) => Vertex::new(xyz),
            PyVertexPosition::Xyzw(xyzw) => Vertex::new(xyzw),
        }
    }
}

/// A vertex for the `Mesh` methods: any `Vertex` position, or the
/// `(position, uv)` and `(position, uv, color)` tuples of `Into<Vertex>`.
#[derive(FromPyObject)]
pub enum PyIntoVertex {
    #[pyo3(annotation = "Vertex | position")]
    Position(PyVertexPosition),
    #[pyo3(annotation = "(position, uv)")]
    Uv([f32; 3], [f32; 2]),
    #[pyo3(annotation = "(position, uv, color)")]
    UvColor([f32; 3], [f32; 2], [f32; 4]),
}

impl From<PyIntoVertex> for Vertex {
    fn from(vertex: PyIntoVertex) -> Self {
        match vertex {
            PyIntoVertex::Position(position) => position.into(),
            PyIntoVertex::Uv(position, uv) => (position, uv).into(),
            PyIntoVertex::UvColor(position, uv, color) => (position, uv, color).into(),
        }
    }
}

/// An instance for the `Mesh` methods: an `Instance`, a `Vertex`
/// whose attributes it copies, or a dict of named attribute values.
#[derive(FromPyObject)]
pub enum PyIntoInstance<'py> {
    #[pyo3(annotation = "Instance")]
    Instance(Instance),
    #[pyo3(annotation = "Vertex")]
    Vertex(Vertex),
    #[pyo3(annotation = "dict[str, float | Sequence[float]]")]
    Attributes(Bound<'py, PyDict>),
}

impl TryFrom<PyIntoInstance<'_>> for Instance {
    type Error = PyErr;

    /// Dict entries are set in insertion order, so their locations follow it.
    fn try_from(value: PyIntoInstance<'_>) -> PyResult<Self> {
        let dict = match value {
            PyIntoInstance::Instance(instance) => return Ok(instance),
            PyIntoInstance::Vertex(vertex) => return Ok(vertex.into()),
            PyIntoInstance::Attributes(dict) => dict,
        };
        let mut instance = Instance::new();
        for (key, value) in dict.iter() {
            let key: String = key
                .extract()
                .map_err(|_| PyTypeError::new_err("instance attribute names must be str"))?;
            let value: PyAttributeValue = value.extract().map_err(|cause| {
                let err = PyTypeError::new_err(format!(
                    "instance attribute '{key}' must be a number or 1 to 4 numbers"
                ));
                err.set_cause(dict.py(), Some(cause));
                err
            })?;
            instance = instance.set(&key, VertexData::try_from(value)?);
        }
        Ok(instance)
    }
}

#[pymethods]
impl Vertex {
    // Attribute-name constants mirror the Rust-side `pub const` declarations
    // on `Vertex` (see `src/mesh/vertex.rs`), exposed to Python as class
    // attributes so example code can write `Vertex.set(Vertex.UV0, ...)`
    // instead of the bare string `"uv0"`. The classattr functions delegate
    // to the same Rust const so the two stay in lock-step.
    //
    // Rust forbids two `pub const NORMAL` items on the same type, so this
    // can't be a second `pub const`; the function-form classattr is the
    // supported pyo3 pattern for re-exporting an existing const.
    #[classattr]
    #[pyo3(name = "NORMAL")]
    fn normal_attr() -> &'static str {
        Self::NORMAL
    }
    #[classattr]
    #[pyo3(name = "TANGENT")]
    fn tangent_attr() -> &'static str {
        Self::TANGENT
    }
    #[classattr]
    #[pyo3(name = "UV0")]
    fn uv0_attr() -> &'static str {
        Self::UV0
    }
    #[classattr]
    #[pyo3(name = "UV1")]
    fn uv1_attr() -> &'static str {
        Self::UV1
    }
    #[classattr]
    #[pyo3(name = "COLOR0")]
    fn color0_attr() -> &'static str {
        Self::COLOR0
    }
    #[classattr]
    #[pyo3(name = "COLOR1")]
    fn color1_attr() -> &'static str {
        Self::COLOR1
    }

    #[new]
    #[lsp_doc("docs/api/geometry/vertex/new.md")]
    pub fn new_py(position: PyVertexPosition) -> Self {
        position.into()
    }

    #[staticmethod]
    #[pyo3(name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_py(position: PyVertexPosition) -> Self {
        // Build the base Vertex from the position, then layer the
        // PBR identity defaults via the same `.set(...)` chain
        // `Vertex::pbr` uses on the Rust side.
        Vertex::from(position)
            .set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0])
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set_py(&self, key: &str, value: PyAttributeValue) -> Result<Self, MeshError> {
        Ok(self.clone().set(key, VertexData::try_from(value)?))
    }

    #[pyo3(name = "create_instance")]
    #[lsp_doc("docs/api/geometry/vertex/create_instance.md")]
    pub fn create_instance_py(&self) -> Instance {
        self.create_instance()
    }
}

#[pymethods]
impl Instance {
    #[new]
    #[lsp_doc("docs/api/geometry/instance/new.md")]
    pub fn new_py() -> Self {
        Self::default()
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/geometry/instance/set.md")]
    pub fn set_py(&self, key: &str, value: PyAttributeValue) -> Result<Self, MeshError> {
        Ok(self.clone().set(key, VertexData::try_from(value)?))
    }
}

#[pymethods]
impl Mesh {
    #[new]
    #[lsp_doc("docs/api/geometry/mesh/new.md")]
    pub fn new_py() -> Self {
        Self::new()
    }

    #[staticmethod]
    #[pyo3(name = "from_vertices")]
    #[lsp_doc("docs/api/geometry/mesh/from_vertices.md")]
    pub fn from_vertices_py(vertices: Vec<PyIntoVertex>) -> Self {
        Mesh::from_vertices(vertices)
    }

    #[pyo3(name = "add_vertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_py(&mut self, vertex: PyIntoVertex) {
        self.add_vertex(vertex);
    }

    #[pyo3(name = "add_vertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_py(&mut self, vertices: Vec<PyIntoVertex>) {
        self.add_vertices(vertices);
    }

    #[pyo3(name = "add_instance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_py(&mut self, item: PyIntoInstance<'_>) -> PyResult<()> {
        self.add_instance(Instance::try_from(item)?);
        Ok(())
    }

    #[pyo3(name = "add_instances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_py(&mut self, items: Vec<PyIntoInstance<'_>>) -> PyResult<()> {
        let instances = items
            .into_iter()
            .map(Instance::try_from)
            .collect::<PyResult<Vec<_>>>()?;
        self.add_instances(instances);
        Ok(())
    }

    #[pyo3(name = "clear_instances")]
    #[lsp_doc("docs/api/geometry/mesh/clear_instances.md")]
    pub fn clear_instances_py(&mut self) {
        self.clear_instances();
    }

    #[pyo3(name = "set_instance_count")]
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count_py(&mut self, n: f64) -> PyResult<()> {
        self.set_instance_count(read_unsigned(n, "Mesh.set_instance_count: n")?);
        Ok(())
    }

    #[pyo3(name = "set_indices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_py(&mut self, indices: Vec<f64>) -> PyResult<()> {
        let read = |(index, number): (usize, f64)| {
            read_unsigned(number, &format!("Mesh.set_indices: item {index}"))
        };
        let indices = indices.into_iter().enumerate().map(read);
        self.set_indices(indices.collect::<PyResult<Vec<u32>>>()?);
        Ok(())
    }

    #[pyo3(name = "clear_indices")]
    #[lsp_doc("docs/api/geometry/mesh/clear_indices.md")]
    pub fn clear_indices_py(&mut self) {
        self.clear_indices();
    }

    // Internal hook for PyRenderable dispatch, not part of the public docs.
    #[doc(hidden)]
    pub fn renderable_type(&self) -> &'static str {
        "Mesh"
    }

    // Internal hook for PyRenderable dispatch, not part of the public docs.
    #[doc(hidden)]
    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        let list = <Self as crate::Renderable>::passes(self);
        crate::PyPassIterator(list.iter().cloned().collect())
    }
}
