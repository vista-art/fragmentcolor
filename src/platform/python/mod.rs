use crate::{FragmentColor, FragmentColorError, Frame, Pass, PyWindowTarget, Renderer, Shader};
use pyo3::prelude::*;

pub mod target;
pub use target::*;

/// FragmentColor expects a RenderCanvas instance to be passed in\
/// Check https://github.com/pygfx/rendercanvas for more information.
#[pymethods]
impl FragmentColor {
    #[staticmethod]
    /// Initializes the Renderer and a RenderTarget compatible with the current platform.
    pub async fn init(window: PyObject) -> Result<(Renderer, PyWindowTarget), FragmentColorError> {}
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FragmentColor>()?;
    m.add_class::<PyWindowTarget>()?;
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    Ok(())
}
