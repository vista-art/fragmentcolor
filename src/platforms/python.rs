use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::error::PythonError;

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

/// Reads a Python number as an unsigned 32-bit integer, such as an index or a count.
/// Negative, fractional, NaN and out-of-range values raise a `FragmentColorError` that names `what`.
pub(crate) fn read_unsigned(value: f64, what: &str) -> PyResult<u32> {
    Ok(super::read_unsigned(value, what)?)
}

impl From<super::NumberError> for PyErr {
    fn from(error: super::NumberError) -> Self {
        crate::error::PyFragmentColorError::new_err(error.to_string())
    }
}
