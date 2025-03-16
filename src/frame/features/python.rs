#![cfg(feature = "python")]

use crate::{Frame, Pass, PyPassIterator};
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

    #[pyo3(name = "add_pass")]
    pub fn add_pass_py(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }

    pub fn renderable_type(&self) -> &'static str {
        "Frame"
    }
}
