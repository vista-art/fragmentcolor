#![cfg(python)]

use crate::SamplerOptions;
use crate::texture::Texture;
use pyo3::prelude::*;

#[pymethods]
impl Texture {
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
}
