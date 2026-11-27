#![cfg(python)]

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::bindings::numbers;
use crate::bindings::python::{
    PyChannels, PyUnsigned, is_list, read_list, read_number, read_numbers, refuse,
};
use crate::{Instance, Mesh, MeshError, Vertex, VertexData};
use lsp_doc::lsp_doc;

/// A vertex or instance attribute from Python:
/// a number, or a sequence of 1 to 4 numbers such as a NumPy array.
/// Sequences come first, so a NumPy array never passes through `float()`.
/// The numbers take the type the shader declares when the mesh attaches to it.
#[derive(FromPyObject)]
pub enum PyAttributeValue<'py> {
    #[pyo3(annotation = "Sequence[float]")]
    Numbers(Vec<f64>),
    #[pyo3(annotation = "float")]
    Number(f64),
    Other(Bound<'py, PyAny>),
}

impl PyAttributeValue<'_> {
    /// The value, or an error that names `label`, as JavaScript reads one.
    /// A list with an item that isn't a number names the item, and any other value raises `TypeError`.
    pub(crate) fn read(self, label: &str) -> PyResult<VertexData> {
        match self {
            PyAttributeValue::Numbers(numbers) => Ok(VertexData::try_from(numbers.as_slice())?),
            PyAttributeValue::Number(number) => Ok(number.into()),
            PyAttributeValue::Other(value) if is_list(&value) => Ok(VertexData::try_from(
                read_numbers(&value, label)?.as_slice(),
            )?),
            PyAttributeValue::Other(value) => Err(refuse(
                &value,
                format_args!("{label} is a number or a list of 1 to 4 numbers"),
            )),
        }
    }
}

/// A mesh error raises the class its `classify` names.
impl From<MeshError> for PyErr {
    fn from(error: MeshError) -> Self {
        crate::errors::raise_error(&error)
    }
}

/// A `Vertex` position from Python: a `Vertex` to copy, a number, or 1 to 4 numbers.
/// [`PyVertexPosition::read`] checks each number with the float vector rule.
#[derive(FromPyObject)]
pub enum PyVertexPosition<'py> {
    #[pyo3(annotation = "Vertex")]
    Vertex(Vertex),
    #[pyo3(
        annotation = "float | Sequence[float] | npt.NDArray[numpy.floating[Any] | numpy.signedinteger[Any] | numpy.uint16 | numpy.uint32 | numpy.uint64]"
    )]
    Numbers(Bound<'py, PyAny>),
}

impl PyVertexPosition<'_> {
    /// The vertex, or an error that names `label`. A value with a length, other
    /// than a string, is a list of numbers, and anything else is one number.
    pub(crate) fn read(&self, label: &str) -> PyResult<Vertex> {
        let value = match self {
            PyVertexPosition::Vertex(vertex) => return Ok(vertex.clone()),
            PyVertexPosition::Numbers(value) => value,
        };
        let numbers = if !value.is_instance_of::<PyString>() && value.len().is_ok() {
            read_numbers(value, label)?
        } else {
            vec![read_number(value, label, 0)?]
        };
        Ok(numbers::read_position(numbers, label)?)
    }
}

/// A vertex for the `Mesh` methods: the `(position, uv)` and `(position, uv, color)`
/// tuples of `Into<Vertex>`, or any `Vertex` position, tried in that order.
#[derive(FromPyObject)]
pub enum PyIntoVertex<'py> {
    #[pyo3(annotation = "(position, uv)")]
    Uv(
        PyChannels<'py>,
        #[pyo3(from_py_with = read_part)] PyChannels<'py>,
    ),
    #[pyo3(annotation = "(position, uv, color)")]
    UvColor(
        PyChannels<'py>,
        #[pyo3(from_py_with = read_part)] PyChannels<'py>,
        #[pyo3(from_py_with = read_part)] PyChannels<'py>,
    ),
    #[pyo3(annotation = "Vertex | position")]
    Position(PyVertexPosition<'py>),
}

/// The uv or color of a vertex tuple: a value with a length, other than a string.
/// A tuple of plain numbers fails here, so it reads as a position, as in `Mesh.add_vertex((0.5, 1.0))`.
fn read_part<'py>(value: &Bound<'py, PyAny>) -> PyResult<PyChannels<'py>> {
    if value.is_instance_of::<PyString>() || value.len().is_err() {
        return Err(PyTypeError::new_err("a vertex part is a list of numbers"));
    }
    Ok(PyChannels::Numbers(value.clone()))
}

impl PyIntoVertex<'_> {
    /// The vertex, each part checked by the float vector rule.
    /// A part's errors name it after `label`, as in `Mesh.add_vertex: vertex: uv`.
    pub(crate) fn read(&self, label: &str) -> PyResult<Vertex> {
        let position = |part: &PyChannels<'_>| part.read_array::<3>(&format!("{label}: position"));
        let uv = |part: &PyChannels<'_>| part.read_array::<2>(&format!("{label}: uv"));
        let color = |part: &PyChannels<'_>| part.read_array::<4>(&format!("{label}: color"));
        match self {
            PyIntoVertex::Uv(first, second) => Ok((position(first)?, uv(second)?).into()),
            PyIntoVertex::UvColor(first, second, third) => {
                Ok((position(first)?, uv(second)?, color(third)?).into())
            }
            PyIntoVertex::Position(position) => position.read(label),
        }
    }
}

/// Reads each vertex of a list, naming a failing one as `{label}: vertex {index}`.
fn read_vertices(vertices: &[PyIntoVertex<'_>], label: &str) -> PyResult<Vec<Vertex>> {
    vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| vertex.read(&format!("{label}: vertex {index}")))
        .collect()
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
    Other(Bound<'py, PyAny>),
}

impl PyIntoInstance<'_> {
    /// The instance, or an error that names `label`, and the key for an attribute it can't read.
    /// Dict entries are set in insertion order, so their locations follow it.
    pub(crate) fn read(self, label: &str) -> PyResult<Instance> {
        let dict = match self {
            PyIntoInstance::Instance(instance) => return Ok(instance),
            PyIntoInstance::Vertex(vertex) => return Ok(vertex.into()),
            PyIntoInstance::Attributes(dict) => dict,
            PyIntoInstance::Other(value) => {
                let forms = "expected an Instance, a Vertex, or a dict of named attributes";
                return Err(refuse(&value, format_args!("{label}: {forms}")));
            }
        };
        let mut instance = Instance::new();
        for (key, value) in dict.iter() {
            let key: String = key
                .extract()
                .map_err(|_| PyTypeError::new_err("instance attribute names must be str"))?;
            let value = value.extract::<PyAttributeValue>()?;
            instance = instance.set(&key, value.read(&format!("{label}: {key}"))?);
        }
        Ok(instance)
    }
}

#[pymethods]
impl Vertex {
    // Attribute-name constants mirror the Rust-side `pub const` declarations
    // on `Vertex` (see `src/geometry/vertex.rs`), exposed to Python as class
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
    pub fn new_py(position: PyVertexPosition<'_>) -> PyResult<Self> {
        position.read("Vertex: position")
    }

    #[staticmethod]
    #[pyo3(name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_py(position: PyVertexPosition<'_>) -> PyResult<Self> {
        // Build the base Vertex from the position, then layer the
        // PBR identity defaults via the same `.set(...)` chain
        // `Vertex::pbr` uses on the Rust side.
        Ok(position
            .read("Vertex.pbr: position")?
            .set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0]))
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set_py(&self, key: &str, value: PyAttributeValue<'_>) -> PyResult<Self> {
        Ok(self.clone().set(key, value.read("Vertex.set: value")?))
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
    pub fn set_py(&self, key: &str, value: PyAttributeValue<'_>) -> PyResult<Self> {
        Ok(self.clone().set(key, value.read("Instance.set: value")?))
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
    pub fn from_vertices_py(vertices: Vec<PyIntoVertex<'_>>) -> PyResult<Self> {
        Ok(Mesh::from_vertices(read_vertices(
            &vertices,
            "Mesh.from_vertices",
        )?))
    }

    #[pyo3(name = "add_vertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_py(&mut self, vertex: PyIntoVertex<'_>) -> PyResult<()> {
        self.add_vertex(vertex.read("Mesh.add_vertex: vertex")?);
        Ok(())
    }

    #[pyo3(name = "add_vertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_py(&mut self, vertices: Vec<PyIntoVertex<'_>>) -> PyResult<()> {
        self.add_vertices(read_vertices(&vertices, "Mesh.add_vertices")?);
        Ok(())
    }

    #[pyo3(name = "add_instance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_py(&mut self, item: PyIntoInstance<'_>) -> PyResult<()> {
        self.add_instance(item.read("Mesh.add_instance")?);
        Ok(())
    }

    #[pyo3(name = "add_instances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_py(&mut self, items: Vec<PyIntoInstance<'_>>) -> PyResult<()> {
        let instances = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| item.read(&format!("Mesh.add_instances: instance {index}")))
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
    pub fn set_instance_count_py(&mut self, n: PyUnsigned<'_>) -> PyResult<()> {
        self.set_instance_count(n.read("Mesh.set_instance_count: n")?);
        Ok(())
    }

    #[pyo3(name = "set_indices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_py(&mut self, indices: Vec<PyUnsigned<'_>>) -> PyResult<()> {
        self.set_indices(read_list(&indices, "Mesh.set_indices")?);
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
