use crate::{Pass, PassObject, Renderable};
use pyo3::prelude::*;
use std::sync::Arc;

// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyPassIterator(pub Vec<Arc<PassObject>>);

impl PyPassIterator {
    pub fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.0.iter().map(|pass| pass.as_ref())
    }
}

impl IntoIterator for PyPassIterator {
    type Item = Arc<PassObject>;
    type IntoIter = std::vec::IntoIter<Arc<PassObject>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[pyclass]
#[derive(Debug, Default)]
/// A Frame represents a graph of Passes that are executed in sequence.
pub struct Frame {
    pub(crate) passes: Vec<Arc<PassObject>>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

#[pymethods]
impl Frame {
    #[new]
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            _dependencies: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: &Pass) {
        self.passes.push(pass.object.clone());
    }

    pub fn passes(&self) -> PyPassIterator {
        let iter = self.passes.iter().map(|pass| pass.clone());

        PyPassIterator(iter.into_iter().collect())
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.passes.iter().map(|pass| pass.as_ref())
    }
}
