#![cfg(python)]

use crate::bindings::Key;
use crate::bindings::python::{
    PyCompareFunction, PyTextureFormat, PyUnsigned, read_bytes, read_list, read_unsigned, refuse,
};
use crate::texture::loading::decode::array_pixels;
use crate::texture::{Mipmap, Texture, TextureError, TextureId, TextureRegion};
use crate::texture::{measure_texel, read_pixels};
use crate::{SamplerOptions, TextureFormat};
use lsp_doc::lsp_doc;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDict, PyList, PyMemoryView, PyTuple};

// The argument types below are `pub` inside this private module,
// so the bindings can take them while they stay out of the crate API.

/// A texture id from Python: a `TextureId` or its integer value, which [`PyTextureId::read`] checks.
#[derive(FromPyObject)]
pub enum PyTextureId<'py> {
    #[pyo3(annotation = "TextureId")]
    Id(TextureId),
    #[pyo3(annotation = "int")]
    Int(Bound<'py, PyAny>),
}

impl PyTextureId<'_> {
    /// The id, or an error that names `label`: a `TypeError` for a value that is neither
    /// a `TextureId` nor an int, and `InvalidInputError` for an int below 0 or past `u64::MAX`.
    pub(crate) fn read(&self, label: &str) -> PyResult<TextureId> {
        let value = match self {
            PyTextureId::Id(id) => return Ok(*id),
            PyTextureId::Int(value) => value,
        };
        match read_unsigned::<u64>(value, label) {
            Ok(id) => Ok(id.into()),
            Err(error) if error.is_instance_of::<PyTypeError>(value.py()) => Err(refuse(
                value,
                format_args!("{label} must be a TextureId or an int"),
            )),
            Err(error) => Err(error),
        }
    }
}

/// Raw texture bytes from Python: a `uint8` ndarray of any shape and layout,
/// or `bytes` or a sequence of ints, tried in that order. [`PyTextureBytes::read`] checks each int.
#[derive(FromPyObject)]
pub enum PyTextureBytes<'py> {
    #[pyo3(annotation = "ndarray[uint8]")]
    Array(numpy::PyReadonlyArrayDyn<'py, u8>),
    #[pyo3(annotation = "bytes | Sequence[int]")]
    Bytes(Bound<'py, PyAny>),
}

impl PyTextureBytes<'_> {
    /// The bytes, or an error that names `label` and the item, as [`read_bytes`] reads them.
    pub(crate) fn read(&self, label: &str) -> PyResult<Vec<u8>> {
        match self {
            PyTextureBytes::Array(array) => Ok(ndarray_bytes(array)),
            PyTextureBytes::Bytes(bytes) => read_bytes(bytes, label),
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

/// What `create_texture` takes, as the `TypeError` for a value it can't read lists it.
pub(crate) const TEXTURE_INPUTS: &str = "bytes (bytes, a bytearray, a memoryview or a list or tuple of ints), a uint8 ndarray, a path, a URL, a Texture or a Mipmap";

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
    Other(Bound<'py, PyAny>),
}

impl PyTextureSource<'_> {
    /// The texture input the source stands for, or an error that names `label`.
    /// An ndarray brings its own size, and each int of a list or tuple must be a byte.
    /// Any other value raises a `TypeError` that lists `inputs`, what the call takes.
    pub(crate) fn read(self, label: &str, inputs: &str) -> PyResult<crate::TextureInput> {
        use crate::TextureData;
        let data = match self {
            PyTextureSource::Texture(texture) => TextureData::CloneOf(texture),
            PyTextureSource::Bytes(bytes) => TextureData::Bytes(bytes.as_bytes().to_vec()),
            PyTextureSource::ByteArray(bytes) => TextureData::Bytes(bytes.to_vec()),
            PyTextureSource::MemoryView(view) => {
                TextureData::Bytes(view.call_method0("tobytes")?.extract()?)
            }
            PyTextureSource::List(list) => TextureData::Bytes(read_bytes(list.as_any(), label)?),
            PyTextureSource::Tuple(tuple) => TextureData::Bytes(read_bytes(tuple.as_any(), label)?),
            PyTextureSource::Source(source) => TextureData::from(source.as_str()),
            PyTextureSource::Path(path) => TextureData::Path(path),
            PyTextureSource::Array(array) => return Ok(read_ndarray(&array, None, None)?),
            PyTextureSource::Mipmap(chain) => TextureData::Prepared(chain),
            PyTextureSource::Other(value) => {
                return Err(refuse(&value, format_args!("{label} takes {inputs}")));
            }
        };
        Ok(data.into())
    }
}

/// The texture input for a `uint8` array, which brings its own size.
/// A `size` keyword must match it, and its channels must fit the `format`, `Rgba` when unset.
pub(crate) fn read_ndarray(
    array: &numpy::PyReadonlyArrayDyn<'_, u8>,
    size: Option<crate::Size>,
    format: Option<TextureFormat>,
) -> Result<crate::TextureInput, crate::texture::TextureError> {
    use numpy::PyUntypedArrayMethods;

    let format = format.unwrap_or_default();
    let read = || ndarray_bytes(array);
    let (pixels, own) = array_pixels(array.shape(), read, format.to_wgpu())?;
    let fits = |size: &crate::Size| {
        (size.width, size.height, size.depth.unwrap_or(1)) == (own.width, own.height, 1)
    };
    if let Some(size) = size.filter(|size| !fits(size)) {
        return Err(crate::texture::TextureError::InvalidInput(format!(
            "size {size:?} differs from the ndarray's {}x{}",
            own.width, own.height
        )));
    }
    Ok((pixels, format, own).into())
}

/// A texture usage from Python: a `TextureUsage`, such as
/// `TextureUsage.COPY_SRC | TextureUsage.TEXTURE_BINDING`, or its bits as an int.
#[derive(FromPyObject)]
pub enum PyTextureUsage<'py> {
    #[pyo3(annotation = "TextureUsage")]
    Usage(crate::TextureUsage),
    #[pyo3(annotation = "int")]
    Bits(PyUnsigned<'py>),
}

impl PyTextureUsage<'_> {
    /// The usage bits, or an error that names `label`: a `TypeError` for a value that is
    /// no usage and no int, and an `InvalidInputError` for an int out of range.
    pub(crate) fn read(&self, label: &str) -> PyResult<u32> {
        let value = match self {
            PyTextureUsage::Usage(usage) => return Ok(usage.bits),
            PyTextureUsage::Bits(PyUnsigned::Int(value)) => value,
        };
        match read_unsigned(value, label) {
            Err(error) if error.is_instance_of::<PyTypeError>(value.py()) => Err(refuse(
                value,
                format_args!("{label} must be a TextureUsage or an int"),
            )),
            bits => bits,
        }
    }
}

/// A `Texture.write_region` region from Python: `[w, h]`, `[x, y, w, h]` or
/// `[x, y, z, w, h, d]` in any sequence of ints, a `ScreenRegion`, or a dict.
#[derive(FromPyObject)]
pub enum PyTextureRegion<'py> {
    Numbers(Vec<PyUnsigned<'py>>),
    #[pyo3(annotation = "ScreenRegion")]
    Screen(crate::ScreenRegion),
    #[pyo3(annotation = "dict[str, int | None]")]
    Layout(Bound<'py, PyDict>),
    Other(Bound<'py, PyAny>),
}

impl TryFrom<PyTextureRegion<'_>> for TextureRegion {
    type Error = PyErr;

    fn try_from(region: PyTextureRegion<'_>) -> Result<Self, Self::Error> {
        let items = match region {
            PyTextureRegion::Numbers(items) => items,
            PyTextureRegion::Screen(screen) => return Ok(TextureRegion::try_from(screen)?),
            PyTextureRegion::Layout(layout) => return layout_region(&layout),
            PyTextureRegion::Other(value) => return Err(refuse(&value, REGION_FORMS)),
        };
        let numbers = read_list(&items, "Texture.write_region")?;
        match *numbers {
            [w, h] => Ok([w, h].into()),
            [x, y, w, h] => Ok([x, y, w, h].into()),
            [x, y, z, w, h, d] => Ok([x, y, z, w, h, d].into()),
            _ => Err(TextureError::InvalidInput(format!(
                "a region takes 2, 4 or 6 numbers, got {}",
                numbers.len()
            ))
            .into()),
        }
    }
}

/// The forms `Texture.write_region` reads a region from, as its `TypeError` lists them.
const REGION_FORMS: &str = "Texture.write_region: a region is [w, h], [x, y, w, h], [x, y, z, w, h, d], a ScreenRegion or a dict with keys such as x, y, width and height";

/// Reads the dict form through the reader the bindings share.
/// A key like `bytes_per_row` also takes its camelCase form, `bytesPerRow`.
fn layout_region(layout: &Bound<'_, PyDict>) -> PyResult<TextureRegion> {
    let read = |(camel, snake): Key| -> PyResult<Option<u32>> {
        for name in [snake, camel] {
            if let Some(value) = layout.get_item(name)? {
                if value.is_none() {
                    return Ok(None);
                }
                return read_unsigned(&value, &format!("Texture.write_region: {snake}")).map(Some);
            }
        }
        Ok(None)
    };
    crate::texture::region::read_region(read)
}

/// `SamplerOptions` from Python: the class itself, or a dict of its fields.
#[derive(FromPyObject)]
pub enum PySamplerOptions<'py> {
    #[pyo3(annotation = "SamplerOptions")]
    Options(SamplerOptions),
    #[pyo3(annotation = "dict[str, bool | CompareFunction | None]")]
    Dict(Bound<'py, PyDict>),
    Other(Bound<'py, PyAny>),
}

impl PySamplerOptions<'_> {
    /// The options for `call`. Missing keys keep the defaults, and unknown keys are ignored.
    /// A value of another type, or a key of the wrong type, raises a `TypeError` that names `call`.
    pub(crate) fn read(self, call: &str) -> PyResult<SamplerOptions> {
        let dict = match self {
            PySamplerOptions::Options(options) => return Ok(options),
            PySamplerOptions::Dict(dict) => dict,
            PySamplerOptions::Other(value) => {
                let forms = "sampler options must be a SamplerOptions or a dict of repeat_x, repeat_y, smooth and compare";
                return Err(refuse(&value, format_args!("{call}: {forms}")));
            }
        };
        let defaults = SamplerOptions::default();
        let flag = |key: &str, default: bool| -> PyResult<bool> {
            match dict.get_item(key)? {
                None => Ok(default),
                Some(value) => value
                    .extract()
                    .map_err(|_| refuse(&value, format_args!("{call}: {key} must be a bool"))),
            }
        };
        Ok(SamplerOptions {
            repeat_x: flag("repeat_x", defaults.repeat_x)?,
            repeat_y: flag("repeat_y", defaults.repeat_y)?,
            smooth: flag("smooth", defaults.smooth)?,
            compare: match dict.get_item("compare")? {
                None => defaults.compare,
                Some(value) if value.is_none() => None,
                Some(value) => {
                    let label = format!("{call}: compare");
                    Some(value.extract::<PyCompareFunction>()?.read(&label)?)
                }
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
        compare = None,
    ))]
    pub fn new_py(
        repeat_x: bool,
        repeat_y: bool,
        smooth: bool,
        compare: Option<PyCompareFunction<'_>>,
    ) -> PyResult<Self> {
        let compare = compare.map(|compare| compare.read("SamplerOptions: compare"));
        Ok(Self {
            repeat_x,
            repeat_y,
            smooth,
            compare: compare.transpose()?.or(SamplerOptions::default().compare),
        })
    }
}

#[pymethods]
impl Mipmap {
    /// Build a chain from `bytes` for `format`. If `size` is `None`, `bytes`
    /// is decoded as an image (PNG / JPEG / etc.); if `size` is provided,
    /// `bytes` is treated as raw pixel data already laid out for `format` at
    /// `size`. Pure CPU work. Call it from a worker thread (e.g.
    /// `ThreadPoolExecutor`) and pass the result to
    /// `renderer.create_texture(chain)` for the GPU upload.
    #[staticmethod]
    #[pyo3(name = "build", signature = (bytes, format, size=None))]
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build_py(
        bytes: PyTextureBytes<'_>,
        format: PyTextureFormat<'_>,
        size: Option<crate::values::PySize<'_>>,
    ) -> pyo3::PyResult<Mipmap> {
        let input = crate::TextureInput {
            data: crate::TextureData::Bytes(bytes.read("Mipmap.build: bytes")?),
            options: crate::TextureOptions {
                size: size.map(|size| size.read("Mipmap.build")).transpose()?,
                format: format.read("Mipmap.build: format")?,
                ..Default::default()
            },
        };
        Ok(Self::build(input)?)
    }

    #[pyo3(name = "format")]
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format_py(&self) -> TextureFormat {
        TextureFormat::from_wgpu(self.format)
    }

    #[pyo3(name = "size")]
    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size_py(&self) -> crate::Size {
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
    pub fn level_py(&self, index: PyUnsigned<'_>) -> pyo3::PyResult<Vec<u8>> {
        Ok(self
            .level(index.read("Mipmap.level: index")? as usize)?
            .to_vec())
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
        self.set_sampler_options(options.read("Texture.set_sampler_options")?);
        Ok(())
    }

    #[pyo3(name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_py(&self, data: PyTextureBytes<'_>) -> pyo3::PyResult<()> {
        let bytes = data.read("Texture.write: data")?;
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
        let bytes = data.read("Texture.write_region: data")?;
        match region {
            Some(region) => self.write_region(&bytes, TextureRegion::try_from(region)?)?,
            None => self.write(&bytes)?,
        }
        Ok(())
    }

    #[pyo3(name = "resize")]
    #[lsp_doc("docs/api/texture/texture/resize.md")]
    pub fn resize_py(&self, size: crate::PySize<'_>) -> pyo3::PyResult<()> {
        self.resize(size.read("Texture.resize")?)?;
        Ok(())
    }

    /// Read back mip 0 as a uint8 ndarray shaped (height, width, bytes per pixel),
    /// with a leading layer axis for array textures. The bytes keep the native
    /// format, so a float texture reads as floats through `image.view(numpy.float16)`
    /// for `Rgba16Float` or `image.view(numpy.float32)` for `Rgba32Float`.
    #[pyo3(name = "get_image")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub fn get_image_py(&self) -> pyo3::PyResult<Py<numpy::PyArrayDyn<u8>>> {
        read_array(self)
    }
}

/// The mip-0 pixels of `texture` as a uint8 ndarray shaped (height, width, bytes per pixel),
/// with a leading layer axis for array textures.
fn read_array(texture: &Texture) -> PyResult<Py<numpy::PyArrayDyn<u8>>> {
    use numpy::PyArrayMethods;
    // One snapshot reads the pixels and their shape, even across a resize.
    let object = texture.object();
    let data = pollster::block_on(read_pixels(&texture.context, &object))?;
    let extent = object.size;
    let texel = measure_texel(object.format)? as usize;
    let (width, height) = (extent.width as usize, extent.height as usize);
    let layers = extent.depth_or_array_layers.max(1) as usize;
    let shape = match layers {
        1 => vec![height, width, texel],
        _ => vec![layers, height, width, texel],
    };
    let expected: usize = shape.iter().product();
    if data.len() != expected {
        return Err(crate::errors::PyFragmentColorError::new_err(format!(
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
