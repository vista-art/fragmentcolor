#![cfg(python)]

use crate::{Frame, Pass, PyPassIterator};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;

#[pymethods]
impl Frame {
    #[new]
    #[lsp_doc("docs/api/core/frame/new.md")]
    pub fn new_py() -> Self {
        Self::new()
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> PyPassIterator {
        let iter = self.passes.iter().cloned();

        PyPassIterator(iter.into_iter().collect())
    }

    #[pyo3(name = "add_pass")]
    #[lsp_doc("docs/api/core/frame/add_pass.md")]
    pub fn add_pass_py(&mut self, pass: &Pass) {
        self.add_pass(pass);
    }

    pub fn renderable_type(&self) -> &'static str {
        "Frame"
    }
}
