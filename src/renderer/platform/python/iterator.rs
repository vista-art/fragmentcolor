use crate::PassObject;
use pyo3::prelude::*;
use std::sync::Arc;

// `from_py_object` opts in to the FromPyObject derive that pyo3 used to
// generate automatically for `#[pyclass]` + `Clone` types; the auto-derive
// is being removed in a future pyo3 release.
#[pyclass(from_py_object)]
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
