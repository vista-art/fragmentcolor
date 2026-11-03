//! `PyRenderable`: what Python hands `render`, a Shader, Pass, Mesh, Scene or a list.

use crate::{Mesh, Pass, PassObject, Renderable, Scene, Shader};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySequence, PyString};
use std::sync::Arc;

const EXPECTED: &str = "Expected a Scene, Pass, Shader, Mesh, or a sequence of them";

/// A renderable built from Python: one object, or a sequence of them.
///
/// It keeps the objects and lists their passes when the renderer asks, so a render
/// refused before it draws runs no setup, such as a Scene's default camera and light.
/// One object renders as Rust renders it, so a Pass brings the passes it requires.
/// A sequence renders as a Rust list does: each item's passes in order,
/// each pass once and after the passes it requires.
pub struct PyRenderable {
    items: Vec<Box<dyn Renderable>>,
}

impl PyRenderable {
    pub fn from_any(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Some(item) = Self::read_object(obj) {
            return Ok(Self { items: vec![item] });
        }
        // A str is a sequence of one-letter strs, never of renderables.
        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err(EXPECTED));
        }
        let Ok(sequence) = obj.cast::<PySequence>() else {
            return Err(PyTypeError::new_err(EXPECTED));
        };
        let mut items = Vec::new();
        for index in 0..sequence.len()? {
            let item = sequence.get_item(index)?;
            let item = Self::read_object(&item).ok_or_else(|| {
                PyTypeError::new_err(format!(
                    "item {index}: expected a Scene, Pass, Shader, or Mesh"
                ))
            })?;
            items.push(item);
        }
        Ok(Self { items })
    }

    fn read_object(object: &Bound<'_, PyAny>) -> Option<Box<dyn Renderable>> {
        if let Ok(pass) = object.cast::<Pass>() {
            return Some(Box::new(pass.borrow().clone()));
        }
        if let Ok(shader) = object.cast::<Shader>() {
            return Some(Box::new(shader.borrow().clone()));
        }
        if let Ok(mesh) = object.cast::<Mesh>() {
            return Some(Box::new(mesh.borrow().clone()));
        }
        if let Ok(scene) = object.cast::<Scene>() {
            return Some(Box::new(scene.borrow().clone()));
        }
        None
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.items.passes()
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.items.roots()
    }

    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        self.items.peek_passes()
    }
}
