use crate::{Pass, PassObject, Renderable};
use std::sync::Arc;
use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod features;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Default)]
#[lsp_doc("docs/api/frame/frame.md")]
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

impl Frame {
    #[lsp_doc("docs/api/frame/constructor.md")]
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            _dependencies: Vec::new(),
        }
    }

    #[lsp_doc("docs/api/frame/add_pass.md")]
    pub fn add_pass(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.passes.iter().map(|p| p.as_ref())
    }
}
