#![cfg(python)]

use crate::bindings::Key;
use crate::bindings::python::{
    PyChannels, PyMesh, PyShader, PyUnsigned, read_list, read_pair, read_unsigned, refuse,
};
use crate::{Pass, PassObject, PassType, ScreenRegion};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use std::sync::Arc;

#[pymethods]
impl Pass {
    #[new]
    #[lsp_doc("docs/api/core/pass/new.md")]
    pub fn new_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Render)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "compute")]
    #[lsp_doc("docs/api/core/pass/compute.md")]
    pub fn compute_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Compute)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "from_shader")]
    #[lsp_doc("docs/api/core/pass/from_shader.md")]
    pub fn from_shader_py(name: &str, shader: PyShader<'_>) -> PyResult<Self> {
        let shader = shader.read("Pass.from_shader: shader")?;
        Ok(Self {
            object: Arc::new(PassObject::from_shader(name, shader.object)),
        })
    }

    // Internal duck-typed interface used by PyRenderable dispatch, not part of public docs.
    #[doc(hidden)]
    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.object.clone()])
    }

    #[pyo3(name = "name")]
    #[lsp_doc("docs/api/core/pass/name.md")]
    pub fn name_py(&self) -> String {
        self.name()
    }

    #[pyo3(name = "load_previous")]
    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous_py(&self) {
        self.load_previous();
    }

    #[pyo3(name = "add_shader")]
    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader_py(&self, shader: PyShader<'_>) -> Result<(), PyErr> {
        Ok(self.add_shader(&shader.read("Pass.add_shader: shader")?)?)
    }

    #[pyo3(name = "add_mesh")]
    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh_py(&self, mesh: PyMesh<'_>) -> Result<(), PyErr> {
        Ok(self.add_mesh(&mesh.read("Pass.add_mesh: mesh")?)?)
    }

    /// Unified `Pass.add`, which branches on the runtime Python type. Adding a
    /// new `SceneObject` Rust-side means adding one extra cast arm below.
    #[pyo3(name = "add")]
    #[lsp_doc("docs/api/core/pass/add.md")]
    pub fn add_py<'py>(
        slf: PyRef<'py, Self>,
        object: &Bound<'py, PyAny>,
    ) -> PyResult<PyRef<'py, Self>> {
        if let Ok(model) = object.cast::<crate::Model>() {
            slf.add(&*model.borrow())?;
        } else if let Ok(camera) = object.cast::<crate::scene::Camera>() {
            slf.add(&*camera.borrow())?;
        } else if let Ok(light) = object.cast::<crate::scene::Light>() {
            slf.add(&*light.borrow())?;
        } else {
            return Err(refuse(
                object,
                "Pass.add: expected a Model, Camera, or Light",
            ));
        }
        Ok(slf)
    }

    #[pyo3(name = "set_clear_color")]
    #[lsp_doc("docs/api/core/pass/set_clear_color.md")]
    pub fn set_clear_color_py(&self, color: PyChannels<'_>) -> PyResult<()> {
        self.object
            .set_clear_color(color.read_color("Pass.set_clear_color")?)?;
        Ok(())
    }

    #[pyo3(name = "set_viewport")]
    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport_py(&self, viewport: PyViewport<'_>) -> PyResult<()> {
        self.object
            .set_viewport(ScreenRegion::try_from(viewport)?)?;
        Ok(())
    }

    #[pyo3(name = "set_compute_dispatch")]
    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch_py(
        &self,
        x: PyUnsigned<'_>,
        y: PyUnsigned<'_>,
        z: PyUnsigned<'_>,
    ) -> PyResult<()> {
        self.object.set_compute_dispatch(
            x.read("Pass.set_compute_dispatch: x")?,
            y.read("Pass.set_compute_dispatch: y")?,
            z.read("Pass.set_compute_dispatch: z")?,
        );
        Ok(())
    }

    #[pyo3(name = "set_target")]
    #[lsp_doc("docs/api/core/pass/set_target.md")]
    pub fn set_target_py(&self, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Try TextureTarget wrapper first
            if let Ok(bound) = target.bind(py).cast::<crate::targets::PyTextureTarget>() {
                let tt = bound.borrow();
                return Ok(self.set_target(&tt.inner)?);
            }
            // Try Texture handle
            if let Ok(tex) = target.bind(py).cast::<crate::texture::Texture>() {
                let t = tex.borrow();
                return Ok(self.set_target(&*t)?);
            }
            Err(refuse(
                target.bind(py),
                "Pass.set_target: target must be a Texture or a TextureTarget",
            ))
        })
    }

    #[pyo3(name = "set_depth_target")]
    #[lsp_doc("docs/api/core/pass/set_depth_target.md")]
    pub fn set_depth_target_py(&self, target: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // Depth textures are Texture handles
            if let Ok(tex) = target.bind(py).cast::<crate::texture::Texture>() {
                let t = tex.borrow();
                return Ok(self.set_depth_target(&*t)?);
            }
            // Or a TextureTarget (if provided)
            if let Ok(bound) = target.bind(py).cast::<crate::targets::PyTextureTarget>() {
                let tt = bound.borrow();
                return Ok(self.set_depth_target(&tt.inner)?);
            }
            Err(refuse(
                target.bind(py),
                "Pass.set_depth_target: target must be a Texture or a TextureTarget",
            ))
        })
    }

    #[pyo3(name = "is_compute")]
    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute_py(&self) -> bool {
        self.object.is_compute()
    }

    #[pyo3(name = "require")]
    #[lsp_doc("docs/api/core/pass/require.md")]
    pub fn require_py(&self, deps: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            let any = deps.bind(py);
            let r = crate::PyRenderable::from_any(any, "Pass.require")?;
            Ok(self.require(&r)?)
        })
    }

    // Internal duck-typed interface used by PyRenderable dispatch, not part of public docs.
    #[doc(hidden)]
    pub fn renderable_type(&self) -> &'static str {
        "Pass"
    }
}

/// A viewport from Python: a `ScreenRegion`, a dict of `x`, `y`, `width` and `height`
/// or of `min_x`, `min_y`, `max_x` and `max_y`, `[x, y, w, h]`, a size `(w, h)` at the origin,
/// or an origin and a size as two pairs, `((x, y), (w, h))`, as Rust reads them.
#[derive(FromPyObject)]
pub enum PyViewport<'py> {
    #[pyo3(annotation = "ScreenRegion")]
    Region(ScreenRegion),
    #[pyo3(annotation = "dict[str, int | None]")]
    Fields(Bound<'py, PyDict>),
    Pairs([Vec<PyUnsigned<'py>>; 2]),
    Numbers(Vec<PyUnsigned<'py>>),
    Other(Bound<'py, PyAny>),
}

/// The forms `Pass.set_viewport` reads a region from, as its `TypeError` lists them.
const VIEWPORT_FORMS: &str = "Pass.set_viewport: a region is [w, h], [x, y, w, h], [[x, y], [w, h]], a dict of x, y, width and height or a ScreenRegion";

/// Reads each number as an int that [`PyUnsigned::read`] checks.
/// Any other value raises a `TypeError` that lists the forms.
impl TryFrom<PyViewport<'_>> for ScreenRegion {
    type Error = PyErr;

    fn try_from(viewport: PyViewport<'_>) -> Result<Self, Self::Error> {
        match viewport {
            PyViewport::Region(region) => Ok(region),
            PyViewport::Fields(fields) => match read_fields(&fields)? {
                Some(region) => Ok(region),
                None => Err(refuse(&fields, VIEWPORT_FORMS)),
            },
            PyViewport::Other(value) => Err(refuse(&value, VIEWPORT_FORMS)),
            PyViewport::Pairs([origin, size]) => {
                let origin = read_pair(&origin, "Pass.set_viewport: origin")?;
                let size = read_pair(&size, "Pass.set_viewport: size")?;
                Ok(ScreenRegion::new(origin, size)?)
            }
            PyViewport::Numbers(items) => {
                let dimensions = read_list(&items, "Pass.set_viewport")?;
                Ok(ScreenRegion::try_from(dimensions.as_slice())?)
            }
        }
    }
}

/// Reads the dict form as JavaScript reads it, each key in either spelling, `minX` first.
/// A key set to `None` counts as missing, and the result is `None` when neither form is complete.
fn read_fields(fields: &Bound<'_, PyDict>) -> PyResult<Option<ScreenRegion>> {
    let read = |(camel, snake): Key| -> PyResult<Option<u32>> {
        for name in [camel, snake] {
            if let Some(value) = fields.get_item(name)?.filter(|value| !value.is_none()) {
                let label = format!("Pass.set_viewport: {name}");
                return read_unsigned(&value, &label).map(Some);
            }
        }
        Ok(None)
    };
    ScreenRegion::read_fields(read)
}
