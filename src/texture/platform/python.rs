#![cfg(python)]

use crate::SamplerOptions;
use crate::texture::{Texture, TextureId, TextureWriteOptions};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;
#[pymethods]
impl TextureWriteOptions {
    #[staticmethod]
    #[pyo3(name = "whole")]
    #[lsp_doc("docs/api/texture_write_options/whole.md")]
    pub fn whole_py() -> Self {
        Self::whole()
    }

    #[pyo3(name = "with_bytes_per_row")]
    #[lsp_doc("docs/api/texture_write_options/with_bytes_per_row.md")]
    pub fn with_bytes_per_row_py(&self, bpr: u32) -> Self {
        self.clone().with_bytes_per_row(bpr)
    }

    #[pyo3(name = "with_rows_per_image")]
    #[lsp_doc("docs/api/texture_write_options/with_rows_per_image.md")]
    pub fn with_rows_per_image_py(&self, rpi: u32) -> Self {
        self.clone().with_rows_per_image(rpi)
    }
}

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

    #[pyo3(name = "write_with")]
    #[lsp_doc("docs/api/core/texture/write_with.md")]
    pub fn write_with_py(
        &self,
        data: pyo3::Py<pyo3::types::PyAny>,
        options: pyo3::Py<pyo3::types::PyAny>,
    ) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let bytes = crate::texture::py_to_texture_bytes(data.bind(py))?;
            let opt = crate::texture::py_to_write_options(options.bind(py))?;
            self.write_with(&bytes, opt)?;
            Ok(())
        })
    }
}
