use thiserror::Error;

// Top-level catch-all error

#[derive(Error, Debug)]
pub enum FragmentColorError {
    #[error(transparent)]
    Shader(#[from] crate::shader::error::ShaderError),
    #[error(transparent)]
    Color(#[from] crate::color::error::ColorError),
    #[error(transparent)]
    Size(#[from] crate::size::error::SizeError),
    #[error(transparent)]
    Pass(#[from] crate::pass::error::PassError),
    #[error(transparent)]
    Frame(#[from] crate::frame::error::FrameError),
    #[error(transparent)]
    Renderer(#[from] crate::renderer::error::RendererError),
    #[error(transparent)]
    Init(#[from] crate::renderer::error::InitializationError),
    #[error(transparent)]
    Display(#[from] crate::target::error::DisplayError),
}

// Python-specific conversions

#[cfg(feature = "python")]
use pyo3::{create_exception, exceptions::PyException, prelude::*};
#[cfg(feature = "python")]
create_exception!(fragment_color, PyFragmentColorError, PyException);

#[cfg(feature = "python")]
impl From<PyErr> for crate::target::error::DisplayError {
    fn from(e: PyErr) -> Self {
        crate::target::error::DisplayError::Error(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<crate::target::error::DisplayError> for PyErr {
    fn from(e: crate::target::error::DisplayError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}
