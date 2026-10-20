#![cfg(python)]

use crate::texture::{Mipmap, Texture, TextureId, TextureRegion};
use crate::{SamplerOptions, TextureFormat};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

// The argument types below are `pub` inside this private module,
// so the bindings can take them while they stay out of the crate API.

/// A texture id from Python: the integer value or a `TextureId`.
#[derive(FromPyObject)]
pub enum PyTextureId {
    #[pyo3(annotation = "int")]
    Int(u64),
    #[pyo3(annotation = "TextureId")]
    Id(TextureId),
}

impl From<PyTextureId> for TextureId {
    fn from(value: PyTextureId) -> Self {
        match value {
            PyTextureId::Int(id) => id.into(),
            PyTextureId::Id(id) => id,
        }
    }
}

/// Raw texture bytes from Python: `bytes` or a sequence of ints,
/// or a contiguous `uint8` ndarray of any shape, tried in that order.
#[derive(FromPyObject)]
pub enum PyTextureBytes<'py> {
    #[pyo3(annotation = "bytes | Sequence[int]")]
    Bytes(Vec<u8>),
    #[pyo3(annotation = "ndarray[uint8]")]
    Array(numpy::PyReadonlyArrayDyn<'py, u8>),
}

impl TryFrom<PyTextureBytes<'_>> for Vec<u8> {
    type Error = PyErr;

    fn try_from(value: PyTextureBytes<'_>) -> Result<Self, Self::Error> {
        match value {
            PyTextureBytes::Bytes(bytes) => Ok(bytes),
            PyTextureBytes::Array(array) => array.as_slice().map(<[u8]>::to_vec).map_err(|_| {
                crate::error::PyFragmentColorError::new_err("ndarray must be contiguous")
            }),
        }
    }
}

/// A `create_texture` source from Python. Raw bytes come only as
/// `bytes` or a `list` of ints, so an int ndarray never reads as bytes.
#[derive(FromPyObject)]
pub enum PyTextureSource<'py> {
    #[pyo3(annotation = "bytes")]
    Bytes(Bound<'py, PyBytes>),
    #[pyo3(annotation = "list[int]")]
    List(Bound<'py, PyList>),
    #[pyo3(annotation = "str")]
    Path(String),
    #[pyo3(annotation = "ndarray[uint8]")]
    Array(numpy::PyReadonlyArrayDyn<'py, u8>),
    #[pyo3(annotation = "Mipmap")]
    Mipmap(Mipmap),
}

/// A `Texture.write_region` region from Python, tried in this order.
#[derive(FromPyObject)]
pub enum PyTextureRegion<'py> {
    #[pyo3(annotation = "[w, h]")]
    Size([u32; 2]),
    #[pyo3(annotation = "[x, y, w, h]")]
    Rect([u32; 4]),
    #[pyo3(annotation = "[x, y, z, w, h, d]")]
    Volume([u32; 6]),
    #[pyo3(annotation = "ScreenRegion")]
    Screen(crate::ScreenRegion),
    #[pyo3(annotation = "dict[str, int | None]")]
    Layout(Bound<'py, PyDict>),
}

impl TryFrom<PyTextureRegion<'_>> for TextureRegion {
    type Error = PyErr;

    fn try_from(region: PyTextureRegion<'_>) -> Result<Self, Self::Error> {
        match region {
            PyTextureRegion::Size(size) => Ok(size.into()),
            PyTextureRegion::Rect(rect) => Ok(rect.into()),
            PyTextureRegion::Volume(volume) => Ok(volume.into()),
            PyTextureRegion::Screen(screen) => Ok(screen.into()),
            PyTextureRegion::Layout(layout) => layout_region(&layout),
        }
    }
}

/// Reads the known keys of the dict form, so any other entry is ignored.
/// A key like `bytes_per_row` also takes its camelCase form, `bytesPerRow`.
fn layout_region(layout: &Bound<'_, PyDict>) -> PyResult<TextureRegion> {
    let read = |names: &[&str]| -> PyResult<Option<u32>> {
        for name in names {
            if let Some(value) = layout.get_item(*name)? {
                return value.extract::<Option<u32>>();
            }
        }
        Ok(None)
    };

    let mut region = if read(&["min_x", "minX"])?.is_some() || read(&["max_x", "maxX"])?.is_some() {
        let min_x = read(&["min_x", "minX"])?.unwrap_or(0);
        let min_y = read(&["min_y", "minY"])?.unwrap_or(0);
        let max_x = read(&["max_x", "maxX"])?.unwrap_or(min_x);
        let max_y = read(&["max_y", "maxY"])?.unwrap_or(min_y);
        let min_z = read(&["min_z", "minZ"])?.unwrap_or(0);
        let max_z = read(&["max_z", "maxZ"])?.unwrap_or(min_z.saturating_add(1));
        TextureRegion::new(
            [min_x, min_y, min_z],
            [
                max_x.saturating_sub(min_x),
                max_y.saturating_sub(min_y),
                max_z.saturating_sub(min_z).max(1),
            ],
        )
    } else {
        let x = read(&["x"])?.unwrap_or(0);
        let y = read(&["y"])?.unwrap_or(0);
        let z = read(&["z"])?.unwrap_or(0);
        let w = read(&["width"])?.unwrap_or(0);
        let h = read(&["height"])?.unwrap_or(0);
        let d = read(&["depth"])?.unwrap_or(1);
        TextureRegion::new([x, y, z], [w, h, d])
    };
    region.bytes_per_row = read(&["bytes_per_row", "bytesPerRow"])?;
    region.rows_per_image = read(&["rows_per_image", "rowsPerImage"])?;
    Ok(region)
}

/// `SamplerOptions` from Python: the class itself, or a dict of its fields.
#[derive(FromPyObject)]
pub enum PySamplerOptions<'py> {
    #[pyo3(annotation = "SamplerOptions")]
    Options(SamplerOptions),
    #[pyo3(annotation = "dict[str, bool | CompareFunction | None]")]
    Dict(Bound<'py, PyDict>),
}

impl TryFrom<PySamplerOptions<'_>> for SamplerOptions {
    type Error = PyErr;

    /// Missing keys keep the defaults, and unknown keys are ignored.
    fn try_from(input: PySamplerOptions<'_>) -> Result<Self, Self::Error> {
        let dict = match input {
            PySamplerOptions::Options(options) => return Ok(options),
            PySamplerOptions::Dict(dict) => dict,
        };
        let defaults = Self::default();
        let flag = |key: &str, default: bool| -> PyResult<bool> {
            match dict.get_item(key)? {
                None => Ok(default),
                Some(value) => value
                    .extract()
                    .map_err(|_| PyTypeError::new_err(format!("'{key}' must be a bool"))),
            }
        };
        Ok(Self {
            repeat_x: flag("repeat_x", defaults.repeat_x)?,
            repeat_y: flag("repeat_y", defaults.repeat_y)?,
            smooth: flag("smooth", defaults.smooth)?,
            compare: match dict.get_item("compare")? {
                None => defaults.compare,
                Some(value) => value.extract().map_err(|_| {
                    PyTypeError::new_err("'compare' must be a CompareFunction or None")
                })?,
            },
        })
    }
}

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
    pub fn set_sampler_options_py(&self, options: PySamplerOptions<'_>) -> pyo3::PyResult<()> {
        self.set_sampler_options(options.try_into()?);
        Ok(())
    }

    #[pyo3(name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_py(&self, data: PyTextureBytes<'_>) -> pyo3::PyResult<()> {
        let bytes: Vec<u8> = data.try_into()?;
        self.write(&bytes)?;
        Ok(())
    }

    #[pyo3(name = "write_region")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
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
