use std::fmt::Display;

use numpy::{PyArrayDescrMethods, PyArrayDyn, dtype};
use pyo3::exceptions::{PyOverflowError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyMemoryView};
use pyo3::wrap_pyfunction;

use crate::error::{InvalidInputError, PythonError};
use crate::platforms::vector;

use crate::{
    Color, Pass, Renderer, ScreenRegion, Shader, Size,
    target::{PyTextureTarget, RenderCanvasFrame, RenderCanvasTarget, rendercanvas_context_hook},
};

/// Python module initializer for fragmentcolor
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<crate::renderer::ErrorKind>()?;

    // Helpers
    m.add_class::<Size>()?;
    m.add_class::<ScreenRegion>()?;

    // Mesh/Vertex bindings
    m.add_class::<crate::mesh::Mesh>()?;
    m.add_class::<crate::mesh::Vertex>()?;
    m.add_class::<crate::mesh::Instance>()?;

    // Mesh primitives
    m.add_class::<crate::mesh::Quad>()?;

    // Scene graph: Scene + Model + Camera + Light + Material + AlphaMode are
    // the v0.12.0 surface for 3D content. Their pyclass impls live next to
    // the Rust types (see `src/scene/platform/python.rs`,
    // `src/material/platform/python.rs`); registering them here makes them
    // importable as `from fragmentcolor import Scene, Model, ...`.
    m.add_class::<crate::scene::Scene>()?;
    m.add_class::<crate::scene::Model>()?;
    m.add_class::<crate::scene::Camera>()?;
    m.add_class::<crate::scene::Light>()?;
    m.add_class::<crate::scene::LightKind>()?;
    m.add_class::<crate::material::Material>()?;
    m.add_class::<crate::material::AlphaMode>()?;
    m.add_class::<crate::material::TextureSlot>()?;

    // Texture handle + format
    m.add_class::<crate::texture::Texture>()?;
    m.add_class::<crate::texture::TextureId>()?;
    m.add_class::<crate::TextureFormat>()?;
    m.add_class::<crate::texture::Mipmap>()?;
    m.add_class::<crate::SamplerOptions>()?;
    m.add_class::<crate::CompareFunction>()?;
    m.add_class::<crate::TextureUsage>()?;

    // TextureTarget (headless) API
    m.add_class::<PyTextureTarget>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;

    // Custom error types: FragmentColorError and its subclasses
    let py = m.py();
    m.add(
        "FragmentColorError",
        py.get_type::<crate::error::PyFragmentColorError>(),
    )?;
    m.add(
        "InvalidInputError",
        py.get_type::<crate::error::InvalidInputError>(),
    )?;
    m.add(
        "UnsupportedMipmapFormatError",
        py.get_type::<crate::error::UnsupportedMipmapFormatError>(),
    )?;
    m.add(
        "MalformedImageError",
        py.get_type::<crate::error::MalformedImageError>(),
    )?;
    m.add(
        "OccludedSurfaceError",
        py.get_type::<crate::error::OccludedSurfaceError>(),
    )?;
    m.add("DeviceError", py.get_type::<crate::error::DeviceError>())?;
    m.add("LimitError", py.get_type::<crate::error::LimitError>())?;
    m.add("FeatureError", py.get_type::<crate::error::FeatureError>())?;
    m.add("BindingError", py.get_type::<crate::error::BindingError>())?;
    m.add("EntryError", py.get_type::<crate::error::EntryError>())?;
    m.add("SceneLoadError", PythonError::SceneLoad.class(py)?)?;
    m.add("LightError", PythonError::Light.class(py)?)?;
    m.add("CameraError", PythonError::Camera.class(py)?)?;
    m.add("MaterialError", PythonError::Material.class(py)?)?;

    Ok(())
}

/// A Python int that a binding reads with [`PyUnsigned::read`], which names the parameter in its errors.
/// Extraction keeps the object as it came, so the stub hints `int` and the check runs where the label is known.
#[derive(FromPyObject)]
pub enum PyUnsigned<'py> {
    #[pyo3(annotation = "int")]
    Int(Bound<'py, PyAny>),
}

impl PyUnsigned<'_> {
    /// The int as a `u32`, or an error that names `label`, as [`read_unsigned`] reads it.
    pub(crate) fn read(&self, label: &str) -> PyResult<u32> {
        let PyUnsigned::Int(value) = self;
        read_unsigned(value, label)
    }
}

/// Reads each int of a list, naming a failing item as `{label}: item {index}`.
pub(crate) fn read_list(items: &[PyUnsigned<'_>], label: &str) -> PyResult<Vec<u32>> {
    items
        .iter()
        .enumerate()
        .map(
            |(index, PyUnsigned::Int(item))| match item.extract::<u32>() {
                Ok(number) if !item.is_instance_of::<PyBool>() => Ok(number),
                _ => read_unsigned(item, &format!("{label}: item {index}")),
            },
        )
        .collect()
}

/// An unsigned integer type that [`read_unsigned`] fills from a Python int.
pub(crate) trait Unsigned:
    for<'a, 'py> FromPyObject<'a, 'py, Error = PyErr> + Display
{
    /// The largest value, which the range error names.
    const MAX: Self;
}

impl Unsigned for u32 {
    const MAX: Self = u32::MAX;
}

impl Unsigned for u64 {
    const MAX: Self = u64::MAX;
}

/// Reads a Python int as an unsigned integer, such as a count, an index, a size or an id.
/// An object with `__index__`, such as a NumPy integer, counts as an int, and a `bool` or any other
/// type raises `TypeError`. An int below 0 or past the type's range raises `InvalidInputError`.
pub(crate) fn read_unsigned<T: Unsigned>(value: &Bound<'_, PyAny>, label: &str) -> PyResult<T> {
    let py = value.py();
    if value.is_instance_of::<PyBool>() {
        return Err(PyTypeError::new_err(format!(
            "{label} must be an int, got bool"
        )));
    }
    match value.extract::<T>() {
        Ok(number) => Ok(number),
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            Err(InvalidInputError::new_err(format!(
                "{label} must be a whole number from 0 to {}, got {}",
                T::MAX,
                show(value)
            )))
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => Err(PyTypeError::new_err(
            format!("{label} must be an int, got {}", type_name(value)),
        )),
        Err(error) => Err(error),
    }
}

/// A color, a vector or another list of floats from Python: any sequence of numbers, NumPy arrays
/// included, except the bytes forms. Its readers name the parameter in their errors.
/// The hint's ndarray leaves out the uint8 and bool arrays the readers refuse.
#[derive(FromPyObject)]
pub enum PyChannels<'py> {
    #[pyo3(
        annotation = "Sequence[float] | npt.NDArray[numpy.floating[Any] | numpy.signedinteger[Any] | numpy.uint16 | numpy.uint32 | numpy.uint64]"
    )]
    Numbers(Bound<'py, PyAny>),
}

impl PyChannels<'_> {
    /// Exactly `N` items, each read by [`read_numbers`] and checked by [`vector::read_floats`].
    pub(crate) fn read_array<const N: usize>(&self, label: &str) -> PyResult<[f32; N]> {
        let PyChannels::Numbers(value) = self;
        Ok(vector::read_vector::<N>(
            read_numbers(value, label)?,
            label,
        )?)
    }

    /// A color from `[r, g, b]` with an opaque alpha or `[r, g, b, a]`, read as [`PyChannels::read_array`] reads it.
    pub(crate) fn read_color(&self, label: &str) -> PyResult<Color> {
        let PyChannels::Numbers(value) = self;
        Ok(vector::read_color(read_numbers(value, label)?, label)?)
    }
}

/// Each item of a Python sequence as an `f64`. A bytes form, a `bool` or a value that isn't
/// a number raises `TypeError`, and an int too large for a float raises `InvalidInputError`.
pub(crate) fn read_numbers(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Vec<f64>> {
    if is_bytes(value) {
        return Err(PyTypeError::new_err(format!(
            "{label} takes numbers, got {}",
            type_name(value)
        )));
    }
    value
        .extract::<Vec<Bound<'_, PyAny>>>()?
        .iter()
        .enumerate()
        .map(|(index, item)| read_number(item, label, index))
        .collect()
}

/// Whether the value is one of the bytes forms: `bytes`, `bytearray`, `memoryview` or a uint8 ndarray.
fn is_bytes(value: &Bound<'_, PyAny>) -> bool {
    value.is_instance_of::<PyBytes>()
        || value.is_instance_of::<PyByteArray>()
        || value.is_instance_of::<PyMemoryView>()
        || value.is_instance_of::<PyArrayDyn<u8>>()
}

/// Whether the value is a Python `bool` or a NumPy `bool_`, which read as numbers but aren't.
fn is_bool(value: &Bound<'_, PyAny>) -> bool {
    value.is_instance_of::<PyBool>()
        || value
            .is_instance(&dtype::<bool>(value.py()).typeobj())
            .unwrap_or(false)
}

/// One item as an `f64`, before the float vector rule checks its value.
pub(crate) fn read_number(value: &Bound<'_, PyAny>, label: &str, index: usize) -> PyResult<f64> {
    let py = value.py();
    if is_bool(value) {
        return Err(PyTypeError::new_err(format!(
            "{label}: item {index} must be a number, got bool"
        )));
    }
    match value.extract::<f64>() {
        Ok(number) => Ok(number),
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            Err(InvalidInputError::new_err(format!(
                "{label}: item {index} must be a finite number that fits a 32-bit float, got {}",
                show(value)
            )))
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => {
            Err(PyTypeError::new_err(format!(
                "{label}: item {index} must be a number, got {}",
                type_name(value)
            )))
        }
        Err(error) => Err(error),
    }
}

/// The value as Python prints it, or a note when it is too long to print.
fn show(value: &Bound<'_, PyAny>) -> String {
    match value.str() {
        Ok(text) => text.to_string(),
        Err(_) => "an int too long to print".to_string(),
    }
}

/// The name of the value's type, such as `float`.
fn type_name(value: &Bound<'_, PyAny>) -> String {
    match value.get_type().name() {
        Ok(name) => name.to_string(),
        Err(_) => "an unknown type".to_string(),
    }
}
