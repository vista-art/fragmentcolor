use crate::{Pass, PassObject, Renderable};
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;

mod features;

// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis
#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Default)]
/// A Frame represents a graph of Passes that are executed in sequence.
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

impl Frame {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            _dependencies: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.passes.iter().map(|pass| pass.as_ref())
    }
}
