#![cfg(python)]

use crate::texture::{Mipmap, Texture, TextureId, TextureRegion};
use crate::{CompareFunction, SamplerOptions, TextureFormat};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDict, PyList, PyMemoryView, PyTuple};

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

/// Raw texture bytes from Python: a `uint8` ndarray of any shape and layout,
/// or `bytes` or a sequence of ints, tried in that order.
#[derive(FromPyObject)]
pub enum PyTextureBytes<'py> {
    #[pyo3(annotation = "ndarray[uint8]")]
    Array(numpy::PyReadonlyArrayDyn<'py, u8>),
    #[pyo3(annotation = "bytes | Sequence[int]")]
    Bytes(Vec<u8>),
}

impl From<PyTextureBytes<'_>> for Vec<u8> {
    fn from(value: PyTextureBytes<'_>) -> Self {
        match value {
            PyTextureBytes::Array(array) => ndarray_bytes(&array),
            PyTextureBytes::Bytes(bytes) => bytes,
        }
    }
}

/// The array's elements in row-major order, whatever its memory layout,
/// so a Fortran-ordered, flipped, or sliced array reads like its C copy.
pub(crate) fn ndarray_bytes(array: &numpy::PyReadonlyArrayDyn<'_, u8>) -> Vec<u8> {
    let view = array.as_array();
    match view.as_slice() {
        Some(bytes) => bytes.to_vec(),
        None => view.iter().copied().collect(),
    }
}

/// A `create_texture` source from Python. Raw bytes come only as `bytes`,
/// `bytearray`, `memoryview`, or a `list` or `tuple` of ints, so an int ndarray
/// never reads as bytes. A `str` is a URL with an http(s) scheme and a path otherwise.
#[derive(FromPyObject)]
pub enum PyTextureSource<'py> {
    #[pyo3(annotation = "Texture")]
    Texture(Texture),
    #[pyo3(annotation = "bytes")]
    Bytes(Bound<'py, PyBytes>),
    #[pyo3(annotation = "bytearray")]
    ByteArray(Bound<'py, PyByteArray>),
    #[pyo3(annotation = "memoryview")]
    MemoryView(Bound<'py, PyMemoryView>),
    #[pyo3(annotation = "list[int]")]
    List(Bound<'py, PyList>),
    #[pyo3(annotation = "tuple[int, ...]")]
    Tuple(Bound<'py, PyTuple>),
    #[pyo3(annotation = "str")]
    Source(String),
    #[pyo3(annotation = "os.PathLike[str]")]
    Path(std::path::PathBuf),
    #[pyo3(annotation = "ndarray[uint8]")]
    Array(numpy::PyReadonlyArrayDyn<'py, u8>),
    #[pyo3(annotation = "Mipmap")]
    Mipmap(Mipmap),
}

/// The texture input a Python source stands for. An ndarray brings its own size.
impl TryFrom<PyTextureSource<'_>> for crate::TextureInput {
    type Error = PyErr;

    fn try_from(source: PyTextureSource<'_>) -> PyResult<Self> {
        use crate::TextureData;
        let data = match source {
            PyTextureSource::Texture(texture) => TextureData::CloneOf(texture),
            PyTextureSource::Bytes(bytes) => TextureData::Bytes(bytes.as_bytes().to_vec()),
            PyTextureSource::ByteArray(bytes) => TextureData::Bytes(bytes.to_vec()),
            PyTextureSource::MemoryView(view) => {
                TextureData::Bytes(view.call_method0("tobytes")?.extract()?)
            }
            PyTextureSource::List(list) => TextureData::Bytes(list.extract()?),
            PyTextureSource::Tuple(tuple) => TextureData::Bytes(tuple.extract()?),
            PyTextureSource::Source(source) => TextureData::from(source.as_str()),
            PyTextureSource::Path(path) => TextureData::Path(path),
            PyTextureSource::Array(array) => {
                let (rgba, size) = ndarray_to_rgba(&array)?;
                return Ok((rgba, size).into());
            }
            PyTextureSource::Mipmap(chain) => TextureData::Prepared(chain),
        };
        Ok(data.into())
    }
}

/// Expands an `(h, w)` or `(h, w, 1 | 3 | 4)` uint8 array to RGBA8 bytes.
fn ndarray_to_rgba(
    array: &numpy::PyReadonlyArrayDyn<'_, u8>,
) -> Result<(Vec<u8>, crate::Size), PyErr> {
    use numpy::PyUntypedArrayMethods;

    let shape = array.shape();
    if shape.len() != 2 && shape.len() != 3 {
        return Err(crate::error::PyFragmentColorError::new_err(format!(
            "ndarray must have 2 or 3 dimensions, got {}",
            shape.len()
        )));
    }
    let h = shape[0] as u32;
    let w = shape[1] as u32;
    let c = if shape.len() == 3 { shape[2] } else { 1 };
    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
    let data = ndarray_bytes(array);
    for y in 0..(h as usize) {
        for x in 0..(w as usize) {
            let src_idx = (y * (w as usize) + x) * c;
            let dst_idx = (y * (w as usize) + x) * 4;
            match c {
                1 => {
                    let v = data[src_idx];
                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[v, v, v, 255]);
                }
                3 => {
                    let r = data[src_idx];
                    let g = data[src_idx + 1];
                    let b = data[src_idx + 2];
                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, 255]);
                }
                4 => {
                    rgba[dst_idx..dst_idx + 4].copy_from_slice(&data[src_idx..src_idx + 4]);
                }
                _ => {
                    return Err(crate::error::PyFragmentColorError::new_err(
                        "ndarray last dimension must be 1, 3, or 4",
                    ));
                }
            }
        }
    }
    Ok((rgba, crate::Size::new(w, h, None)))
}

/// A texture usage from Python: a `TextureUsage`, such as
/// `TextureUsage.COPY_SRC | TextureUsage.TEXTURE_BINDING`, or its bits as an int.
#[derive(FromPyObject)]
pub enum PyTextureUsage {
    #[pyo3(annotation = "TextureUsage")]
    Usage(crate::TextureUsage),
    #[pyo3(annotation = "int")]
    Bits(u32),
}

impl From<PyTextureUsage> for u32 {
    fn from(usage: PyTextureUsage) -> Self {
        match usage {
            PyTextureUsage::Usage(usage) => usage.bits,
            PyTextureUsage::Bits(bits) => bits,
        }
    }
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
impl SamplerOptions {
    // A value type for `Texture.set_sampler_options`, which documents it.
    // Swift and Kotlin build it as a record, so it has no page of its own.
    #[doc(hidden)]
    #[new]
    #[pyo3(signature = (
        repeat_x = SamplerOptions::default().repeat_x,
        repeat_y = SamplerOptions::default().repeat_y,
        smooth = SamplerOptions::default().smooth,
        compare = SamplerOptions::default().compare,
    ))]
    pub fn new_py(
        repeat_x: bool,
        repeat_y: bool,
        smooth: bool,
        compare: Option<CompareFunction>,
    ) -> Self {
        Self {
            repeat_x,
            repeat_y,
            smooth,
            compare,
        }
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
            data: crate::TextureData::Bytes(bytes.into()),
            options: crate::TextureOptions {
                size: size.map(crate::Size::try_from).transpose()?,
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
    #[lsp_doc("docs/api/texture/mipmap/level.md")]
    pub fn level_py(&self, index: usize) -> pyo3::PyResult<Vec<u8>> {
        Ok(self.level(index)?.to_vec())
    }

    /// Every mip level's bytes as a list, level 0 first.
    #[pyo3(name = "levels")]
    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn levels_py(&self) -> Vec<Vec<u8>> {
        self.levels().to_vec()
    }
}

#[pymethods]
impl Texture {
    #[pyo3(name = "id")]
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id_py(&self) -> TextureId {
        *self.id()
    }

    #[pyo3(name = "size")]
    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size_py(&self) -> crate::Size {
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
        let bytes: Vec<u8> = data.into();
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
        let bytes: Vec<u8> = data.into();
        let region = match region {
            Some(region) => TextureRegion::try_from(region)?,
            None => TextureRegion::default(),
        };
        self.write_region(&bytes, region)?;
        Ok(())
    }

    /// Read back mip 0 as a uint8 ndarray shaped (height, width, bytes per pixel),
    /// with a leading layer axis for array textures. The bytes keep the native
    /// format, so a float texture reads as floats through `image.view(numpy.float32)`.
    #[pyo3(name = "get_image")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub fn get_image_py(&self) -> pyo3::PyResult<Py<numpy::PyArrayDyn<u8>>> {
        use numpy::PyArrayMethods;
        // One snapshot reads the pixels and their shape, even across a resize.
        let object = self.object();
        let data = pollster::block_on(crate::texture::read_pixels(&self.context, &object))?;
        let extent = object.size;
        let bpp = crate::texture::bytes_per_pixel(object.format) as usize;
        let (width, height) = (extent.width as usize, extent.height as usize);
        let layers = extent.depth_or_array_layers.max(1) as usize;
        let shape = match layers {
            1 => vec![height, width, bpp],
            _ => vec![layers, height, width, bpp],
        };
        let expected: usize = shape.iter().product();
        if data.len() != expected {
            return Err(crate::error::PyFragmentColorError::new_err(format!(
                "Unexpected image data length: expected {expected}, got {}",
                data.len()
            )));
        }
        Python::attach(|py| {
            // The readback is row-major, so it takes the image shape without a copy.
            let image = numpy::PyArray1::from_vec(py, data).reshape(shape)?;
            Ok(image.unbind())
        })
    }
}
