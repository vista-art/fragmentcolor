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
/// A sequence renders its passes in order, and a Pass in it renders on its own, as in a Rust list.
pub struct PyRenderable {
    items: Vec<Item>,
}

/// One object of a `PyRenderable`.
enum Item {
    /// A Scene, Shader, Mesh or Pass that renders as it does in Rust.
    Whole(Box<dyn Renderable>),
    /// A Pass in a sequence: the pass itself, without the passes it requires.
    Bare(Arc<PassObject>),
}

impl Item {
    /// The passes of this item, listed by `list`, one of the `Renderable` listings.
    fn collect(&self, list: fn(&dyn Renderable) -> Arc<[Arc<PassObject>]>) -> Vec<Arc<PassObject>> {
        match self {
            Item::Whole(renderable) => list(renderable.as_ref()).to_vec(),
            Item::Bare(pass) => vec![pass.clone()],
        }
    }
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
            let item = match item.cast::<Pass>() {
                Ok(pass) => Item::Bare(pass.borrow().object.clone()),
                Err(_) => Self::read_object(&item).ok_or_else(|| {
                    PyTypeError::new_err(format!(
                        "item {index}: expected a Scene, Pass, Shader, or Mesh"
                    ))
                })?,
            };
            items.push(item);
        }
        Ok(Self { items })
    }

    fn read_object(obj: &Bound<'_, PyAny>) -> Option<Item> {
        if let Ok(pass) = obj.cast::<Pass>() {
            return Some(Item::Whole(Box::new(pass.borrow().clone())));
        }
        if let Ok(shader) = obj.cast::<Shader>() {
            return Some(Item::Whole(Box::new(shader.borrow().clone())));
        }
        if let Ok(mesh) = obj.cast::<Mesh>() {
            return Some(Item::Whole(Box::new(mesh.borrow().clone())));
        }
        if let Ok(scene) = obj.cast::<Scene>() {
            return Some(Item::Whole(Box::new(scene.borrow().clone())));
        }
        None
    }

    /// Every item's passes, listed by `list`.
    fn gather(
        &self,
        list: fn(&dyn Renderable) -> Arc<[Arc<PassObject>]>,
    ) -> Arc<[Arc<PassObject>]> {
        self.items
            .iter()
            .flat_map(|item| item.collect(list))
            .collect()
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.gather(|renderable| renderable.passes())
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.gather(|renderable| renderable.roots())
    }

    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        self.gather(|renderable| renderable.peek_passes())
    }
}
