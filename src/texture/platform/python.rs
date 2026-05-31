#![cfg(python)]

use crate::SamplerOptions;
use crate::texture::{Mipmap, Texture, TextureId};
use crate::{Size, TextureFormat};
use lsp_doc::lsp_doc;
use pyo3::prelude::*;

#[pymethods]
impl Mipmap {
    /// Build a chain from `bytes` for `format`. If `size` is `None`, `bytes`
    /// is decoded as an image (PNG / JPEG / etc.); if `size` is provided,
    /// `bytes` is treated as raw pixel data already laid out for `format` at
    /// `size`. Pure CPU work — call from a worker thread (e.g.
    /// `ThreadPoolExecutor`) and pass the result to
    /// `renderer.create_texture(chain)` for the GPU upload.
    #[staticmethod]
    #[pyo3(name = "build", signature = (bytes, format, size=None))]
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build_py(
        bytes: pyo3::Py<pyo3::types::PyAny>,
        format: TextureFormat,
        size: Option<crate::size::PySize>,
    ) -> pyo3::PyResult<Mipmap> {
        Python::attach(|py| -> pyo3::PyResult<Mipmap> {
            let bytes = crate::texture::py_to_texture_bytes(bytes.bind(py))?;
            let size: Option<Size> = size.map(Into::into);
            let input = crate::TextureInput {
                data: crate::TextureData::Bytes(bytes),
                options: crate::TextureOptions {
                    size,
                    format,
                    ..Default::default()
                },
            };
            Ok(Self::build(input)?)
        })
    }

    #[pyo3(name = "format")]
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format_py(&self) -> TextureFormat {
        self.format.into()
    }

    #[pyo3(name = "size")]
    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size_py(&self) -> (u32, u32) {
        self.size()
    }

    #[pyo3(name = "count")]
    #[lsp_doc("docs/api/texture/mipmap/count.md")]
    pub fn count_py(&self) -> u32 {
        self.count() as u32
    }

    /// Return the bytes for a single mip level. Use `count()` to discover
    /// the valid range.
    #[pyo3(name = "level")]
    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn level_py(&self, index: u32) -> pyo3::PyResult<Vec<u8>> {
        let levels = self.levels();
        let idx = index as usize;
        if idx >= levels.len() {
            return Err(crate::error::PyFragmentColorError::new_err(format!(
                "level {} out of range; chain has {} levels",
                idx,
                levels.len()
            )));
        }
        Ok(levels[idx].clone())
    }
}

#[pymethods]
impl Texture {
    #[pyo3(name = "id")]
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id_py(&self) -> TextureId {
        self.id
    }

    /// Python property: size -> returns a Size object
    #[getter]
    #[pyo3(name = "size")]
    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size_prop(&self) -> crate::Size {
        self.size()
    }

    /// Python method: aspect() -> f32
    #[pyo3(name = "aspect")]
    #[lsp_doc("docs/api/texture/texture/aspect.md")]
    pub fn aspect_py(&self) -> f32 {
        self.aspect()
    }

    /// Accepts a dict with keys repeat_x, repeat_y, smooth, compare
    #[pyo3(name = "set_sampler_options")]
    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    pub fn set_sampler_options_py(
        &self,
        options: pyo3::Py<pyo3::types::PyAny>,
    ) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let any = options.bind(py);
            let opts = if let Ok(d) = any.cast::<pyo3::types::PyDict>() {
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
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_py(&self, data: pyo3::Py<pyo3::types::PyAny>) -> pyo3::PyResult<()> {
        pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
            let bytes = crate::texture::py_to_texture_bytes(data.bind(py))?;
            self.write(&bytes)?;
            Ok(())
        })
    }

    #[pyo3(name = "write_region")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
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

    /// Read back the mip-0 contents of this texture as tightly-packed bytes
    /// in the texture's native format. Blocks the Python thread synchronously
    /// (via `pollster::block_on`) — the async readback is the canonical
    /// implementation; the blocking wrapper exists because Python does not
    /// have a native async runtime that integrates with the GPU device loop.
    #[pyo3(name = "get_image")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub fn get_image_py(&self) -> pyo3::PyResult<Vec<u8>> {
        pollster::block_on(self.get_image())
            .map_err(|e| crate::error::PyFragmentColorError::new_err(e.to_string()))
    }
}
