use pyo3::exceptions::{PyOverflowError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes};
use pyo3::wrap_pyfunction;

use crate::error::{InvalidInputError, PythonError};

use crate::{
    Pass, Renderer, ScreenRegion, Shader, Size,
    target::{PyTextureTarget, RenderCanvasFrame, RenderCanvasTarget, rendercanvas_context_hook},
};

/// Python module initializer for fragmentcolor
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;

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
    m.add("SceneLoadError", PythonError::SceneLoad.class(py)?)?;
    m.add("LightError", PythonError::Light.class(py)?)?;
    m.add("CameraError", PythonError::Camera.class(py)?)?;

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
                Ok(number) => Ok(number),
                Err(_) => read_unsigned(item, &format!("{label}: item {index}")),
            },
        )
        .collect()
}

/// Reads a Python int as an unsigned 32-bit integer, such as a count, an index or a size.
/// An object with `__index__`, such as a NumPy integer, counts as an int, and any other type
/// raises `TypeError`. An int below 0 or past `u32::MAX` raises `InvalidInputError`, however large.
pub(crate) fn read_unsigned(value: &Bound<'_, PyAny>, label: &str) -> PyResult<u32> {
    let py = value.py();
    match value.extract::<u32>() {
        Ok(number) => Ok(number),
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            Err(InvalidInputError::new_err(format!(
                "{label} must be a whole number from 0 to {}, got {}",
                u32::MAX,
                show(value)
            )))
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => Err(PyTypeError::new_err(
            format!("{label} must be an int, got {}", type_name(value)),
        )),
        Err(error) => Err(error),
    }
}

/// A color or other list of channels from Python: any sequence of numbers, NumPy arrays
/// included, except a bytes object. [`PyChannels::read`] names the parameter in its errors.
#[derive(FromPyObject)]
pub enum PyChannels<'py> {
    #[pyo3(annotation = "Sequence[float]")]
    Numbers(Bound<'py, PyAny>),
}

impl PyChannels<'_> {
    /// Each channel as an `f32`. A bytes object or a value that isn't a number raises
    /// `TypeError`, and a number past the 32-bit float range raises `InvalidInputError`.
    pub(crate) fn read(&self, label: &str) -> PyResult<Vec<f32>> {
        let PyChannels::Numbers(value) = self;
        if value.is_instance_of::<PyBytes>() || value.is_instance_of::<PyByteArray>() {
            return Err(PyTypeError::new_err(format!(
                "{label} takes numbers, got {}",
                type_name(value)
            )));
        }
        value
            .extract::<Vec<Bound<'_, PyAny>>>()?
            .iter()
            .enumerate()
            .map(|(index, item)| read_float(item, label, index))
            .collect()
    }

    /// Exactly `N` channels, read as [`PyChannels::read`] reads them.
    pub(crate) fn read_array<const N: usize>(&self, label: &str) -> PyResult<[f32; N]> {
        let channels = self.read(label)?;
        match <[f32; N]>::try_from(channels.as_slice()) {
            Ok(array) => Ok(array),
            Err(_) => Err(InvalidInputError::new_err(format!(
                "{label} takes {N} numbers, got {}",
                channels.len()
            ))),
        }
    }
}

/// One channel as an `f32`. NaN and infinity pass, and the call decides what to do with them.
fn read_float(value: &Bound<'_, PyAny>, label: &str, index: usize) -> PyResult<f32> {
    let py = value.py();
    let number = match value.extract::<f64>() {
        Ok(number) => number,
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            return Err(InvalidInputError::new_err(format!(
                "{label}: item {index} must fit a 32-bit float, got {}",
                show(value)
            )));
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => {
            return Err(PyTypeError::new_err(format!(
                "{label}: item {index} must be a number, got {}",
                type_name(value)
            )));
        }
        Err(error) => return Err(error),
    };
    let channel = number as f32;
    if number.is_finite() && !channel.is_finite() {
        return Err(InvalidInputError::new_err(format!(
            "{label}: item {index} must fit a 32-bit float, got {number}"
        )));
    }
    Ok(channel)
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
