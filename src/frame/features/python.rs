#![cfg(feature = "python")]

use crate::{Frame, PyPassIterator};
use pyo3::prelude::*;

#[pymethods]
impl Frame {
    #[new]
    pub fn new_py() -> Self {
        Self::new()
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> PyPassIterator {
        let iter = self.passes.iter().map(|pass| pass.clone());

        PyPassIterator(iter.into_iter().collect())
    }

    pub fn renderable_type(&self) -> &'static str {
        "Frame"
    }
}
