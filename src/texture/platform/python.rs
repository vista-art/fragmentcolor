#![cfg(python)]

use crate::TextureFormat;
use crate::texture::PySamplerOptions;
use crate::texture::region::PyTextureRegion;
use crate::texture::{Mipmap, PyTextureBytes, Texture, TextureId, TextureRegion};
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
    #[allow(private_interfaces)]
    pub fn build_py(
        bytes: PyTextureBytes<'_>,
        format: TextureFormat,
        size: Option<crate::size::PySize>,
    ) -> pyo3::PyResult<Mipmap> {
        let input = crate::TextureInput {
            data: crate::TextureData::Bytes(bytes.try_into()?),
            options: crate::TextureOptions {
                size: size.map(Into::into),
                format,
                ..Default::default()
            },
        };
        Ok(Self::build(input)?)
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

    /// Accepts `SamplerOptions` or a dict with keys repeat_x, repeat_y, smooth, compare
    #[pyo3(name = "set_sampler_options")]
    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    #[allow(private_interfaces)]
    pub fn set_sampler_options_py(&self, options: PySamplerOptions<'_>) -> pyo3::PyResult<()> {
        self.set_sampler_options(options.try_into()?);
        Ok(())
    }

    #[pyo3(name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    #[allow(private_interfaces)]
    pub fn write_py(&self, data: PyTextureBytes<'_>) -> pyo3::PyResult<()> {
        let bytes: Vec<u8> = data.try_into()?;
        self.write(&bytes)?;
        Ok(())
    }

    #[pyo3(name = "write_region")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    #[allow(private_interfaces)]
    pub fn write_region_py(
        &self,
        data: PyTextureBytes<'_>,
        region: Option<PyTextureRegion<'_>>,
    ) -> pyo3::PyResult<()> {
        let bytes: Vec<u8> = data.try_into()?;
        let region = match region {
            Some(region) => TextureRegion::try_from(region)?,
            None => TextureRegion::default(),
        };
        self.write_region(&bytes, region)?;
        Ok(())
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
