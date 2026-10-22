use crate::{Mesh, Pass, PassObject, Renderable, Scene, Shader};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySequence, PyString};
use std::sync::Arc;

const EXPECTED: &str = "Expected a Scene, Pass, Shader, Mesh, or a sequence of them";

/// A renderable built from Python: one object, or a sequence of them.
///
/// It keeps the passes to render and the roots that `Pass.require` links.
/// One object renders as Rust renders it, so a Pass brings the passes it requires.
/// A sequence renders its passes in order, and a Pass in it renders on its own, as in a Rust list.
pub struct PyRenderable {
    passes: Vec<Arc<PassObject>>,
    roots: Vec<Arc<PassObject>>,
}

impl PyRenderable {
    pub fn from_any(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Some(renderable) = Self::from_object(obj) {
            return Ok(renderable);
        }
        // A str is a sequence of one-letter strs, never of renderables.
        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err(EXPECTED));
        }
        let Ok(sequence) = obj.cast::<PySequence>() else {
            return Err(PyTypeError::new_err(EXPECTED));
        };
        let mut list = Self {
            passes: Vec::new(),
            roots: Vec::new(),
        };
        for index in 0..sequence.len()? {
            let item = sequence.get_item(index)?;
            let part = match item.cast::<Pass>() {
                Ok(pass) => Self::from_pass(&pass.borrow()),
                Err(_) => Self::from_object(&item).ok_or_else(|| {
                    PyTypeError::new_err(format!(
                        "item {index}: expected a Scene, Pass, Shader, or Mesh"
                    ))
                })?,
            };
            list.passes.extend(part.passes);
            list.roots.extend(part.roots);
        }
        Ok(list)
    }

    fn from_object(obj: &Bound<'_, PyAny>) -> Option<Self> {
        if let Ok(pass) = obj.cast::<Pass>() {
            return Some(Self::from_renderable(&*pass.borrow()));
        }
        if let Ok(shader) = obj.cast::<Shader>() {
            return Some(Self::from_renderable(&*shader.borrow()));
        }
        if let Ok(mesh) = obj.cast::<Mesh>() {
            return Some(Self::from_renderable(&*mesh.borrow()));
        }
        if let Ok(scene) = obj.cast::<Scene>() {
            return Some(Self::from_renderable(&*scene.borrow()));
        }
        None
    }

    fn from_renderable(renderable: &impl Renderable) -> Self {
        Self {
            passes: renderable.passes().to_vec(),
            roots: renderable.roots().to_vec(),
        }
    }

    /// A Pass in a sequence: the pass itself, without the passes it requires.
    fn from_pass(pass: &Pass) -> Self {
        Self {
            passes: vec![pass.object.clone()],
            roots: vec![pass.object.clone()],
        }
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.passes.clone().into()
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.roots.clone().into()
    }
}
