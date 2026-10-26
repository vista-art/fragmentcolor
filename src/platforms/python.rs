use pyo3::exceptions::PyOverflowError;
use pyo3::prelude::*;
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
    m.add("SceneLoadError", PythonError::SceneLoad.class(py)?)?;
    m.add("LightError", PythonError::Light.class(py)?)?;
    m.add("CameraError", PythonError::Camera.class(py)?)?;
    m.add("MaterialError", PythonError::Material.class(py)?)?;

    Ok(())
}

/// Reads a Python int as an unsigned 32-bit integer, such as a count, an index or a size.
/// An object with `__index__`, such as a NumPy integer, counts as an int, and any other type
/// raises `TypeError`. An int below 0 or past `u32::MAX` raises `InvalidInputError`, however large.
pub(crate) fn read_unsigned(value: &Bound<'_, PyAny>) -> PyResult<u32> {
    match value.extract::<u32>() {
        Err(error) if error.is_instance_of::<PyOverflowError>(value.py()) => {
            let shown = match value.str() {
                Ok(text) => text.to_string(),
                Err(_) => "an int too long to print".to_string(),
            };
            Err(InvalidInputError::new_err(format!(
                "expected an int from 0 to {}, got {shown}",
                u32::MAX
            )))
        }
        result => result,
    }
}

/// Reads `None`, or an int as [`read_unsigned`] does.
pub(crate) fn read_optional(value: &Bound<'_, PyAny>) -> PyResult<Option<u32>> {
    if value.is_none() {
        return Ok(None);
    }
    read_unsigned(value).map(Some)
}

/// Reads a pair of ints, such as an origin or a size, as [`read_unsigned`] does.
pub(crate) fn read_pair(value: &Bound<'_, PyAny>) -> PyResult<(u32, u32)> {
    let (first, second) = value.extract::<(Bound<'_, PyAny>, Bound<'_, PyAny>)>()?;
    Ok((read_unsigned(&first)?, read_unsigned(&second)?))
}

/// Reads a sequence of exactly `N` numbers, such as an offset or a scale.
/// A value that isn't a sequence of numbers raises `TypeError`,
/// and a sequence of another length raises `InvalidInputError`.
pub(crate) fn read_floats<const N: usize>(value: &Bound<'_, PyAny>) -> PyResult<[f32; N]> {
    let numbers: Vec<f32> = value.extract()?;
    let count = numbers.len();
    let Ok(floats) = <[f32; N]>::try_from(numbers) else {
        return Err(InvalidInputError::new_err(format!(
            "expected {N} numbers, got {count}"
        )));
    };
    Ok(floats)
}

/// Reads a sequence of ints, such as a list or a NumPy array, as [`read_unsigned`] does.
pub(crate) fn read_list(value: &Bound<'_, PyAny>) -> PyResult<Vec<u32>> {
    value
        .extract::<Vec<Bound<'_, PyAny>>>()?
        .iter()
        .map(read_unsigned)
        .collect()
}
