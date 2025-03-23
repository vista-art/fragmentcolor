use crate::PassObject;
use pyo3::prelude::*;
use std::sync::Arc;

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
