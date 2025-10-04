use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::{
    Frame, Pass, Region, Renderer, Shader, Size,
    target::{PyTextureTarget, RenderCanvasFrame, RenderCanvasTarget, rendercanvas_context_hook},
};

/// Python module initializer for fragmentcolor
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    // Helpers
    m.add_class::<Size>()?;
    m.add_class::<Region>()?;

    // Mesh/Vertex bindings
    m.add_class::<crate::mesh::Mesh>()?;
    m.add_class::<crate::mesh::Vertex>()?;
    m.add_class::<crate::mesh::Instance>()?;
    m.add_class::<crate::mesh::PyVertexValue>()?;

    // Mesh primitives
    m.add_class::<crate::mesh::Quad>()?;

    // Texture handle + format
    m.add_class::<crate::texture::Texture>()?;
    m.add_class::<crate::TextureFormat>()?;

    // TextureTarget (headless) API
    m.add_class::<PyTextureTarget>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<crate::error::PyFragmentColorError>(),
    )?;

    Ok(())
}
