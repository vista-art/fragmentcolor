//! The `fragmentcolor` Python module, and the readers that turn Python values into Rust ones.

use std::fmt::Display;

use numpy::{PyArrayDescrMethods, PyArrayDyn, dtype};
use pyo3::exceptions::{PyOverflowError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyByteArray, PyBytes, PyMemoryView, PySequence, PyString};
use pyo3::wrap_pyfunction;

use crate::bindings::{name_class, numbers};
use crate::errors::{InvalidInputError, PythonError};

use crate::{
    AlphaMode, Camera, Color, CompareFunction, Light, LogLevel, Mesh, Pass, Renderer, ScreenRegion,
    Shader, Size, TextureFormat, Vertex,
    targets::{PyTextureTarget, RenderCanvasFrame, RenderCanvasTarget, rendercanvas_context_hook},
};

/// Python module initializer for fragmentcolor
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<crate::ErrorKind>()?;
    m.add_class::<crate::LogLevel>()?;

    // Helpers
    m.add_class::<Size>()?;
    m.add_class::<ScreenRegion>()?;

    // Mesh/Vertex bindings
    m.add_class::<crate::Mesh>()?;
    m.add_class::<crate::Vertex>()?;
    m.add_class::<crate::Instance>()?;

    // Mesh primitives
    m.add_class::<crate::Quad>()?;

    // Scene graph: Scene + Model + Camera + Light + the material types are
    // the surface for 3D content. Their pyclass impls live next to the Rust
    // types (see `src/scene/platform/python.rs`,
    // `src/scene/pbr_material/platform/python.rs`); registering them here makes them
    // importable as `from fragmentcolor import Scene, Model, ...`. `Material` is a
    // Rust enum, so Python gets the material object itself.
    m.add_class::<crate::scene::Scene>()?;
    m.add_class::<crate::scene::Model>()?;
    m.add_class::<crate::scene::Camera>()?;
    m.add_class::<crate::scene::Light>()?;
    m.add_class::<crate::scene::LightKind>()?;
    m.add_class::<crate::PbrMaterial>()?;
    m.add_class::<crate::UnlitMaterial>()?;
    m.add_class::<crate::ToonMaterial>()?;
    m.add_class::<crate::MaterialMap>()?;
    m.add_class::<crate::AlphaMode>()?;

    // Texture handle + format
    m.add_class::<crate::texture::Texture>()?;
    m.add_class::<crate::texture::TextureId>()?;
    m.add_class::<crate::TextureFormat>()?;
    m.add_class::<crate::texture::Mipmap>()?;
    m.add_class::<crate::SamplerOptions>()?;
    m.add_class::<crate::SamplerInfo>()?;
    m.add_class::<crate::CompareFunction>()?;
    m.add_class::<crate::TextureUsage>()?;

    // TextureTarget (headless) API
    m.add_class::<PyTextureTarget>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;

    // Custom error types: FragmentColorError and its subclasses
    let py = m.py();
    m.add(
        "FragmentColorError",
        py.get_type::<crate::errors::PyFragmentColorError>(),
    )?;
    m.add(
        "InvalidInputError",
        py.get_type::<crate::errors::InvalidInputError>(),
    )?;
    m.add(
        "UnsupportedMipmapFormatError",
        py.get_type::<crate::errors::UnsupportedMipmapFormatError>(),
    )?;
    m.add(
        "MalformedImageError",
        py.get_type::<crate::errors::MalformedImageError>(),
    )?;
    m.add(
        "OccludedSurfaceError",
        py.get_type::<crate::errors::OccludedSurfaceError>(),
    )?;
    m.add("DeviceError", py.get_type::<crate::errors::DeviceError>())?;
    m.add("LimitError", py.get_type::<crate::errors::LimitError>())?;
    m.add("FeatureError", py.get_type::<crate::errors::FeatureError>())?;
    m.add("BindingError", py.get_type::<crate::errors::BindingError>())?;
    m.add("EntryError", py.get_type::<crate::errors::EntryError>())?;
    m.add("SceneLoadError", PythonError::SceneLoad.class(py)?)?;
    m.add("LightError", PythonError::Light.class(py)?)?;
    m.add("CameraError", PythonError::Camera.class(py)?)?;
    m.add("MaterialError", PythonError::Material.class(py)?)?;

    Ok(())
}

/// A Python int that a binding reads with [`PyUnsigned::read`], which names the parameter in its errors.
/// Extraction keeps the object as it came, so the stub hints `int` and the check runs where the label is known.
#[derive(FromPyObject)]
pub enum PyUnsigned<'py> {
    #[pyo3(annotation = "int")]
    Int(Bound<'py, PyAny>),
}

impl PyUnsigned<'_> {
    /// The int as a `u32`, or an error that names `label`, as [`read_unsigned`] reads it.
    pub(crate) fn read(&self, label: &str) -> PyResult<u32> {
        let PyUnsigned::Int(value) = self;
        read_unsigned(value, label)
    }
}

/// A Python float that a binding reads with [`PyFloat::read`], which names the parameter in its errors.
/// Extraction keeps the object as it came, so a `bool`, which pyo3 would read as 1.0, can be refused.
#[derive(FromPyObject)]
pub enum PyFloat<'py> {
    #[pyo3(annotation = "float")]
    Number(Bound<'py, PyAny>),
}

impl PyFloat<'_> {
    /// The number as an `f32`, as JavaScript's `read_float` reads one, or a `TypeError` that
    /// names `label` for a `bool`, a NumPy `bool_` or a value that isn't a number.
    pub(crate) fn read(&self, label: &str) -> PyResult<f32> {
        let PyFloat::Number(value) = self;
        Ok(read_scalar(value, label)? as f32)
    }

    /// The number as [`PyFloat::read`] reads it, for a Material factor or a UV rotation.
    /// NaN, an infinity or a number past the `f32` range raises `InvalidInputError` that names `label`.
    pub(crate) fn read_finite(&self, label: &str) -> PyResult<f32> {
        let PyFloat::Number(value) = self;
        Ok(numbers::read_finite(read_scalar(value, label)?, label)?)
    }
}

/// Reads each int of a list, naming a failing item as `{label}: item {index}`.
pub(crate) fn read_list(items: &[PyUnsigned<'_>], label: &str) -> PyResult<Vec<u32>> {
    items
        .iter()
        .enumerate()
        .map(|(index, PyUnsigned::Int(item))| read_item(item, label, index))
        .collect()
}

/// A pair of ints from any sequence of two, such as an origin or a size, named `label`.
/// Each item reads as [`read_list`] reads it, and any count but two raises `InvalidInputError`,
/// as JavaScript throws for one.
pub(crate) fn read_pair(items: &[PyUnsigned<'_>], label: &str) -> PyResult<(u32, u32)> {
    match read_list(items, label)?[..] {
        [first, second] => Ok((first, second)),
        ref numbers => Err(InvalidInputError::new_err(format!(
            "{label} takes two numbers, got {}",
            numbers.len()
        ))),
    }
}

/// One int of a list as `T`, naming a failing item as `{label}: item {index}`.
/// The label is written only for an item that fails, so a long list costs no strings.
fn read_item<T: Unsigned>(item: &Bound<'_, PyAny>, label: &str, index: usize) -> PyResult<T> {
    match item.extract::<T>() {
        Ok(number) if !item.is_instance_of::<PyBool>() => Ok(number),
        _ => read_unsigned(item, &format!("{label}: item {index}")),
    }
}

/// An unsigned integer type that [`read_unsigned`] fills from a Python int.
pub(crate) trait Unsigned:
    for<'object, 'py> FromPyObject<'object, 'py, Error = PyErr> + Display
{
    const MAX: Self;
}

impl Unsigned for u8 {
    const MAX: Self = u8::MAX;
}

impl Unsigned for u32 {
    const MAX: Self = u32::MAX;
}

impl Unsigned for u64 {
    const MAX: Self = u64::MAX;
}

/// Reads a Python int as an unsigned integer, such as a count, an index, a size or an id.
/// An object with `__index__`, such as a NumPy integer, counts as an int, and a `bool` or any other
/// type raises `TypeError`. An int below 0 or past the type's range raises `InvalidInputError`.
pub(crate) fn read_unsigned<T: Unsigned>(value: &Bound<'_, PyAny>, label: &str) -> PyResult<T> {
    let py = value.py();
    if value.is_instance_of::<PyBool>() {
        return Err(PyTypeError::new_err(format!(
            "{label} must be an int, got bool"
        )));
    }
    match value.extract::<T>() {
        Ok(number) => Ok(number),
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            Err(InvalidInputError::new_err(format!(
                "{label} must be a whole number from 0 to {}, got {}",
                T::MAX,
                show(value)
            )))
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => Err(PyTypeError::new_err(
            format!("{label} must be an int, got {}", name_type(value)),
        )),
        Err(error) => Err(error),
    }
}

/// A Python bool, which a binding reads with [`PyFlag::read`].
/// Its `Other` variant keeps a value of any other type, which the stub leaves out,
/// so the reader raises a `TypeError` that names the parameter where pyo3's message wouldn't.
#[derive(FromPyObject)]
pub enum PyFlag<'py> {
    Value(bool),
    Other(Bound<'py, PyAny>),
}

impl PyFlag<'_> {
    /// The bool, or a `TypeError` that names `label` for a value of another type.
    pub(crate) fn read(&self, label: &str) -> PyResult<bool> {
        match self {
            PyFlag::Value(value) => Ok(*value),
            PyFlag::Other(value) => Err(refuse_flag(value, label)),
        }
    }
}

/// A Python str, which a binding reads with [`PyText::read`].
/// Its `Other` variant keeps a value of any other type, which the stub leaves out,
/// so the reader raises a `TypeError` that names the parameter where pyo3's message wouldn't.
#[derive(FromPyObject)]
pub enum PyText<'py> {
    Value(Bound<'py, PyString>),
    Other(Bound<'py, PyAny>),
}

impl PyText<'_> {
    /// The text, or a `TypeError` that names `label` for a value of another type.
    pub(crate) fn read(self, label: &str) -> PyResult<String> {
        match self {
            PyText::Value(text) => read_text(&text),
            PyText::Other(value) => Err(refuse(&value, format_args!("{label} must be a str"))),
        }
    }
}

/// The text of a str, read as JavaScript reads the same string:
/// each lone surrogate becomes U+FFFD, and a pair of surrogates the character it encodes.
fn read_text(text: &Bound<'_, PyString>) -> PyResult<String> {
    if let Ok(text) = text.to_cow() {
        return Ok(text.into_owned());
    }
    let encoded = text.call_method1("encode", ("utf-16-le", "surrogatepass"))?;
    let (units, _) = encoded.cast::<PyBytes>()?.as_bytes().as_chunks::<2>();
    let units: Vec<u16> = units.iter().map(|unit| u16::from_le_bytes(*unit)).collect();
    Ok(String::from_utf16_lossy(&units))
}

/// A bool, as [`PyFlag::read`] reads one, for a `from_py_with` reader.
/// Such a parameter keeps a literal default, such as `True`, that `help()` can print.
pub(crate) fn read_flag(value: &Bound<'_, PyAny>, label: &str) -> PyResult<bool> {
    value.extract().map_err(|_| refuse_flag(value, label))
}

/// A `TypeError` for a value that is no bool, such as `Model.set_visible: visible must be a bool, got str`.
fn refuse_flag(value: &Bound<'_, PyAny>, label: &str) -> PyErr {
    refuse(value, format_args!("{label} must be a bool"))
}

/// A color, a vector or another list of floats from Python: any sequence of numbers, NumPy arrays
/// included, except the bytes forms. Its readers name the parameter in their errors.
/// The hint's ndarray leaves out the uint8 and bool arrays the readers refuse.
#[derive(FromPyObject)]
pub enum PyChannels<'py> {
    #[pyo3(
        annotation = "Sequence[float] | npt.NDArray[numpy.floating[Any] | numpy.signedinteger[Any] | numpy.uint16 | numpy.uint32 | numpy.uint64]"
    )]
    Numbers(Bound<'py, PyAny>),
}

impl PyChannels<'_> {
    /// Exactly `N` items, each read by [`read_numbers`] and checked by [`numbers::read_floats`].
    pub(crate) fn read_array<const N: usize>(&self, label: &str) -> PyResult<[f32; N]> {
        let PyChannels::Numbers(value) = self;
        Ok(numbers::read_vector::<N>(
            read_numbers(value, label)?,
            label,
        )?)
    }

    /// A color from `[r, g, b]` with an opaque alpha or `[r, g, b, a]`, read as [`PyChannels::read_array`] reads it.
    pub(crate) fn read_color(&self, label: &str) -> PyResult<Color> {
        let PyChannels::Numbers(value) = self;
        Ok(numbers::read_color(read_numbers(value, label)?, label)?)
    }
}

/// Each item of a Python sequence as an `f64`. A value that is no list, a `bool` or an item that
/// isn't a number raises `TypeError`, and an int too large for a float raises `InvalidInputError`.
pub(crate) fn read_numbers(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Vec<f64>> {
    read_items(value, label)?
        .iter()
        .enumerate()
        .map(|(index, item)| read_number(item, label, index))
        .collect()
}

/// The items of a list of numbers, such as a Python list or an ndarray.
/// A bytes form or any other value raises a `TypeError` that names `label`.
pub(crate) fn read_items<'py>(
    value: &Bound<'py, PyAny>,
    label: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    if is_bytes(value) {
        return Err(refuse(value, format_args!("{label} takes numbers")));
    }
    match value.extract::<Vec<Bound<'py, PyAny>>>() {
        Ok(items) => Ok(items),
        Err(error) if !error.is_instance_of::<PyTypeError>(value.py()) => Err(error),
        Err(_) => Err(refuse(
            value,
            format_args!("{label} takes a list or an ndarray of numbers"),
        )),
    }
}

/// The bytes of a `bytes`, `bytearray` or `memoryview`, or of a sequence of ints, each read by
/// [`read_unsigned`] as a byte and named `{label}: item {index}`. A float or a `bool` raises
/// `TypeError`, and an int outside 0 to 255 raises `InvalidInputError`.
pub(crate) fn read_bytes(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Vec<u8>> {
    if let Ok(bytes) = value.cast::<PyBytes>() {
        return Ok(bytes.as_bytes().to_vec());
    }
    if let Ok(bytes) = value.cast::<PyByteArray>() {
        return Ok(bytes.to_vec());
    }
    if value.is_instance_of::<PyMemoryView>() {
        return value.call_method0("tobytes")?.extract();
    }
    let sequence = match value.cast::<PySequence>() {
        Ok(sequence) if !value.is_instance_of::<PyString>() => sequence,
        _ => {
            return Err(PyTypeError::new_err(format!(
                "{label} takes bytes or a sequence of ints, got {}",
                name_type(value)
            )));
        }
    };
    let mut bytes = Vec::with_capacity(sequence.len()?);
    for (index, item) in sequence.try_iter()?.enumerate() {
        bytes.push(read_item::<u8>(&item?, label, index)?);
    }
    Ok(bytes)
}

/// Whether the value is one of the bytes forms: `bytes`, `bytearray`, `memoryview` or a uint8 ndarray.
fn is_bytes(value: &Bound<'_, PyAny>) -> bool {
    value.is_instance_of::<PyBytes>()
        || value.is_instance_of::<PyByteArray>()
        || value.is_instance_of::<PyMemoryView>()
        || value.is_instance_of::<PyArrayDyn<u8>>()
}

/// Whether the value is a Python `bool`, a NumPy `bool_` or a bool ndarray, such as the
/// 0-d `np.array(True)`, which read as numbers but aren't.
fn is_bool(value: &Bound<'_, PyAny>) -> bool {
    value.is_instance_of::<PyBool>()
        || value.is_instance_of::<PyArrayDyn<bool>>()
        || value
            .is_instance(&dtype::<bool>(value.py()).typeobj())
            .unwrap_or(false)
}

/// One item as an `f64`, before the float vector rule checks its value.
pub(crate) fn read_number(value: &Bound<'_, PyAny>, label: &str, index: usize) -> PyResult<f64> {
    read_scalar(value, format_args!("{label}: item {index}"))
}

/// One number as an `f64`. A `bool` or a value that isn't a number raises `TypeError`, and an int
/// too large for a float raises `InvalidInputError`, each naming `label`, which is written only then.
fn read_scalar(value: &Bound<'_, PyAny>, label: impl Display) -> PyResult<f64> {
    let py = value.py();
    if is_bool(value) {
        return Err(PyTypeError::new_err(format!(
            "{label} must be a number, got bool"
        )));
    }
    match value.extract::<f64>() {
        Ok(number) => Ok(number),
        Err(error) if error.is_instance_of::<PyOverflowError>(py) => {
            Err(InvalidInputError::new_err(format!(
                "{label} must be a finite number that fits a 32-bit float, got {}",
                show(value)
            )))
        }
        Err(error) if error.is_instance_of::<PyTypeError>(py) => Err(PyTypeError::new_err(
            format!("{label} must be a number, got {}", name_type(value)),
        )),
        Err(error) => Err(error),
    }
}

/// The value as Python prints it, or a note when it is too long to print.
fn show(value: &Bound<'_, PyAny>) -> String {
    match value.str() {
        Ok(text) => text.to_string(),
        Err(_) => "an int too long to print".to_string(),
    }
}

/// A `TypeError` for a value the call can't take.
/// Its message reads `{expected}, got {type}`, as JavaScript's does.
/// An argument enum keeps such a value in a last `Bound<PyAny>` or `Py<PyAny>` variant,
/// which the stub leaves out, so its reader raises this in place of pyo3's message, which names the enum.
pub(crate) fn refuse(value: &Bound<'_, PyAny>, expected: impl Display) -> PyErr {
    PyTypeError::new_err(format!("{expected}, got {}", name_type(value)))
}

/// A `TypeError` for a value that is no `class`, such as `Pass.add_shader: shader must be a Shader, got int`.
pub(crate) fn refuse_class(value: &Bound<'_, PyAny>, label: &str, class: &str) -> PyErr {
    refuse(value, format_args!("{label} must be {}", name_class(class)))
}

/// Declares the argument type of a parameter that takes one class.
///
/// Its `Other` variant keeps a value of any other type, which the stub leaves out,
/// so `read` raises a `TypeError` that names the call where pyo3's message wouldn't.
/// The class is read by type alone, so a borrow conflict raises its own error.
macro_rules! take_class {
    ($name:ident, $class:ident) => {
        /// An argument that takes one class, or any other value for `read` to refuse.
        #[derive(pyo3::FromPyObject)]
        pub enum $name<'py> {
            Object(pyo3::Bound<'py, $class>),
            Other(pyo3::Bound<'py, pyo3::PyAny>),
        }

        impl $name<'_> {
            /// The object, or a `TypeError` that names `label` for a value of another type.
            pub(crate) fn read(self, label: &str) -> pyo3::PyResult<$class> {
                match self {
                    Self::Object(object) => Ok(object.try_borrow()?.clone()),
                    Self::Other(value) => Err($crate::bindings::python::refuse_class(
                        &value,
                        label,
                        stringify!($class),
                    )),
                }
            }
        }
    };
}

take_class!(PyMesh, Mesh);
take_class!(PyShader, Shader);
take_class!(PyVertex, Vertex);
take_class!(PyPass, Pass);
take_class!(PyCamera, Camera);
take_class!(PyLight, Light);
take_class!(PyAlphaMode, AlphaMode);
take_class!(PyTextureFormat, TextureFormat);
take_class!(PyLogLevel, LogLevel);
take_class!(PyCompareFunction, CompareFunction);

/// Declares the argument type of a list parameter.
///
/// Its `Other` variant keeps a value that is no list, which the stub leaves out,
/// so `items` raises a `TypeError` that names the call where pyo3's message wouldn't.
/// Each item reads as its own type, which names a failing item.
macro_rules! take_list {
    ($name:ident, $item:ident, $things:literal) => {
        /// A list argument, or any other value for `items` to refuse.
        #[derive(pyo3::FromPyObject)]
        pub enum $name<'py> {
            Items(Vec<$item<'py>>),
            Other(pyo3::Bound<'py, pyo3::PyAny>),
        }

        impl<'py> $name<'py> {
            /// The items, or a `TypeError` that names `label` for a value that is no list.
            pub(crate) fn items(self, label: &str) -> pyo3::PyResult<Vec<$item<'py>>> {
                match self {
                    Self::Items(items) => Ok(items),
                    Self::Other(value) => Err($crate::bindings::python::refuse(
                        &value,
                        format_args!(concat!("{} must be a list of ", $things), label),
                    )),
                }
            }
        }
    };
}
pub(crate) use take_list;

take_list!(PyMeshes, PyMesh, "meshes");
take_list!(PyPasses, PyPass, "passes");
take_list!(PyIndices, PyUnsigned, "ints");

/// Whether the value is a list of items: a sequence other than a `str` or a bytes form.
pub(crate) fn is_list(value: &Bound<'_, PyAny>) -> bool {
    !value.is_instance_of::<PyString>() && !is_bytes(value) && value.cast::<PySequence>().is_ok()
}

/// The name of the value's type, such as `float`.
fn name_type(value: &Bound<'_, PyAny>) -> String {
    match value.get_type().name() {
        Ok(name) => name.to_string(),
        Err(_) => "an unknown type".to_string(),
    }
}
