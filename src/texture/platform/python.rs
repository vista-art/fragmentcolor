#![cfg(python)]

use crate::SamplerOptions;
use crate::texture::{Texture, TextureId};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;

#[pymethods]
impl Texture {
    #[pyo3(name = "id")]
    #[lsp_doc("docs/api/core/texture/id.md")]
    pub fn id_py(&self) -> TextureId {
        self.id
    }

    /// Python property: size -> returns a Size object
    #[getter]
    #[pyo3(name = "size")]
    pub fn size_prop(&self) -> crate::Size {
        self.size()
    }

    /// Python method: aspect() -> f32
    #[pyo3(name = "aspect")]
    pub fn aspect_py(&self) -> f32 {
        self.aspect()
    }

    /// Accepts a dict with keys repeat_x, repeat_y, smooth, compare
    #[pyo3(name = "set_sampler_options")]
    pub fn set_sampler_options_py(
        &self,
        options: pyo3::Py<pyo3::types::PyAny>,
    ) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let any = options.bind(py);
            let opts = if let Ok(d) = any.downcast::<pyo3::types::PyDict>() {
                let mut o = SamplerOptions::default();
                if let Some(v) = d.get_item("repeat_x")? {
                    o.repeat_x = v.extract()?;
                }
                if let Some(v) = d.get_item("repeat_y")? {
                    o.repeat_y = v.extract()?;
                }
                if let Some(v) = d.get_item("smooth")? {
                    o.smooth = v.extract()?;
                }
                if let Some(v) = d.get_item("compare")? {
                    if v.is_none() {
                        o.compare = None;
                    } else {
                        o.compare = Some(v.extract()?);
                    }
                }
                o
            } else {
                any.extract::<SamplerOptions>()?
            };
            self.set_sampler_options(opts);
            Ok(())
        })
    }

    #[pyo3(name = "write")]
    #[lsp_doc("docs/api/core/texture/write.md")]
    pub fn write_py(&self, data: pyo3::Py<pyo3::types::PyAny>) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let bytes = crate::texture::py_to_texture_bytes(data.bind(py))?;
            self.write(&bytes)?;
            Ok(())
        })
    }

    #[pyo3(name = "write_region")]
    #[lsp_doc("docs/api/core/texture/write_region.md")]
    pub fn write_region_py(
        &self,
        data: pyo3::Py<pyo3::types::PyAny>,
        region: pyo3::Py<pyo3::types::PyAny>,
    ) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let bytes = crate::texture::py_to_texture_bytes(data.bind(py))?;
            let r = crate::texture::region::py_to_texture_region(region.bind(py))?;
            self.write_region(&bytes, r)?;
            Ok(())
        })
    }
}
