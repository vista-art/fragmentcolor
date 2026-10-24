use super::error::MeshError;
use crate::shader::is_whole_in;
use wgpu::VertexFormat;

/// A vertex or instance attribute value, as `Vertex::set` and `Instance::set` take it.
///
/// An `f32` or a `u32` keeps its type and must match the shader input.
/// Any other number, an unsuffixed literal or an `f64` or `i32` value included,
/// takes the type the shader declares at its location when the mesh attaches to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VertexData {
    F32(f32),
    F32x2([f32; 2]),
    F32x3([f32; 3]),
    F32x4([f32; 4]),
    U32(u32),
    U32x2([u32; 2]),
    U32x3([u32; 3]),
    U32x4([u32; 4]),
    Number(f64),
    Number2([f64; 2]),
    Number3([f64; 3]),
    Number4([f64; 4]),
}

macro_rules! impl_from {
    ($($source:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$source> for VertexData {
                fn from(value: $source) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

impl_from!(
    f32 => F32,
    [f32; 2] => F32x2,
    [f32; 3] => F32x3,
    [f32; 4] => F32x4,
    u32 => U32,
    [u32; 2] => U32x2,
    [u32; 3] => U32x3,
    [u32; 4] => U32x4,
    f64 => Number,
    [f64; 2] => Number2,
    [f64; 3] => Number3,
    [f64; 4] => Number4,
);

impl From<[f32; 1]> for VertexData {
    fn from([value]: [f32; 1]) -> Self {
        Self::F32(value)
    }
}

impl From<[u32; 1]> for VertexData {
    fn from([value]: [u32; 1]) -> Self {
        Self::U32(value)
    }
}

impl From<[f64; 1]> for VertexData {
    fn from([value]: [f64; 1]) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for VertexData {
    fn from(value: i32) -> Self {
        Self::Number(value.into())
    }
}

impl From<[i32; 1]> for VertexData {
    fn from([value]: [i32; 1]) -> Self {
        Self::Number(value.into())
    }
}

impl From<[i32; 2]> for VertexData {
    fn from(values: [i32; 2]) -> Self {
        Self::Number2(values.map(f64::from))
    }
}

impl From<[i32; 3]> for VertexData {
    fn from(values: [i32; 3]) -> Self {
        Self::Number3(values.map(f64::from))
    }
}

impl From<[i32; 4]> for VertexData {
    fn from(values: [i32; 4]) -> Self {
        Self::Number4(values.map(f64::from))
    }
}

/// One to four plain numbers, the form every binding sends.
impl TryFrom<&[f64]> for VertexData {
    type Error = MeshError;

    fn try_from(values: &[f64]) -> Result<Self, Self::Error> {
        match *values {
            [value] => Ok(Self::Number(value)),
            [x, y] => Ok(Self::Number2([x, y])),
            [x, y, z] => Ok(Self::Number3([x, y, z])),
            [x, y, z, w] => Ok(Self::Number4([x, y, z, w])),
            _ => Err(MeshError::Length(values.len())),
        }
    }
}

/// How many numbers an untyped value holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Count {
    One,
    Two,
    Three,
    Four,
}

impl Count {
    pub(crate) fn len(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    /// Whether `format` has this many components, of a type plain numbers can fill.
    pub(crate) fn fits(self, format: VertexFormat) -> bool {
        split(format).is_some_and(|(_, len)| len == self.len())
    }

    /// The float format an attribute of this many numbers takes until a shader types it.
    pub(crate) fn float_format(self) -> VertexFormat {
        match self {
            Self::One => VertexFormat::Float32,
            Self::Two => VertexFormat::Float32x2,
            Self::Three => VertexFormat::Float32x3,
            Self::Four => VertexFormat::Float32x4,
        }
    }
}

/// What a value fixes about its attribute's format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    /// A typed Rust value, which fixes the format.
    Typed(VertexFormat),
    /// Plain numbers, which take the format a shader declares.
    Numbers(Count),
}

/// Why a value doesn't fit a format: the format's WGSL type and what the value holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Misfit {
    pub(crate) expected: &'static str,
    pub(crate) found: String,
}

impl Misfit {
    /// Explains the misfit of attribute `name` at `place`, such as `vertex 2`.
    pub(crate) fn explain(&self, name: &str, place: &str) -> String {
        format!(
            "{name}: expected {}, got {} at {place}",
            self.expected, self.found
        )
    }
}

/// The scalar type every component of a vertex format shares.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scalar {
    Float,
    Uint,
    Sint,
}

/// Up to four 32-bit components, as the bit patterns the GPU reads.
struct Components {
    bits: [u32; 4],
    len: usize,
}

impl Components {
    fn new(values: &[u32]) -> Self {
        let mut bits = [0; 4];
        bits[..values.len()].copy_from_slice(values);
        Self {
            bits,
            len: values.len(),
        }
    }
}

impl VertexData {
    /// What this value fixes about its attribute's format.
    pub(crate) fn shape(&self) -> Shape {
        match self {
            Self::F32(_) => Shape::Typed(VertexFormat::Float32),
            Self::F32x2(_) => Shape::Typed(VertexFormat::Float32x2),
            Self::F32x3(_) => Shape::Typed(VertexFormat::Float32x3),
            Self::F32x4(_) => Shape::Typed(VertexFormat::Float32x4),
            Self::U32(_) => Shape::Typed(VertexFormat::Uint32),
            Self::U32x2(_) => Shape::Typed(VertexFormat::Uint32x2),
            Self::U32x3(_) => Shape::Typed(VertexFormat::Uint32x3),
            Self::U32x4(_) => Shape::Typed(VertexFormat::Uint32x4),
            Self::Number(_) => Shape::Numbers(Count::One),
            Self::Number2(_) => Shape::Numbers(Count::Two),
            Self::Number3(_) => Shape::Numbers(Count::Three),
            Self::Number4(_) => Shape::Numbers(Count::Four),
        }
    }

    /// The format this value packs as: the one a shader `declared` for its attribute,
    /// else its own when typed, else floats.
    pub(crate) fn resolve(&self, declared: Option<VertexFormat>) -> VertexFormat {
        match (declared, self.shape()) {
            (Some(format), _) | (None, Shape::Typed(format)) => format,
            (None, Shape::Numbers(count)) => count.float_format(),
        }
    }

    /// Checks that this value fits `format`.
    pub(crate) fn check(&self, format: VertexFormat) -> Result<(), Misfit> {
        self.convert(format).map(drop)
    }

    /// Appends this value's bytes in `format` to `out`, or says why it doesn't fit.
    pub(crate) fn write(&self, format: VertexFormat, out: &mut Vec<u8>) -> Result<(), Misfit> {
        let components = self.convert(format)?;
        for bits in &components.bits[..components.len] {
            out.extend_from_slice(&bits.to_ne_bytes());
        }
        Ok(())
    }

    /// Bytes that tell two values apart for vertex dedup: the variant, then its exact bits.
    pub(crate) fn to_key(self) -> Vec<u8> {
        let (tag, bytes): (u8, &[u8]) = match &self {
            Self::F32(value) => (0, bytemuck::bytes_of(value)),
            Self::F32x2(values) => (1, bytemuck::cast_slice(values)),
            Self::F32x3(values) => (2, bytemuck::cast_slice(values)),
            Self::F32x4(values) => (3, bytemuck::cast_slice(values)),
            Self::U32(value) => (4, bytemuck::bytes_of(value)),
            Self::U32x2(values) => (5, bytemuck::cast_slice(values)),
            Self::U32x3(values) => (6, bytemuck::cast_slice(values)),
            Self::U32x4(values) => (7, bytemuck::cast_slice(values)),
            Self::Number(value) => (8, bytemuck::bytes_of(value)),
            Self::Number2(values) => (9, bytemuck::cast_slice(values)),
            Self::Number3(values) => (10, bytemuck::cast_slice(values)),
            Self::Number4(values) => (11, bytemuck::cast_slice(values)),
        };
        let mut key = Vec::with_capacity(bytes.len() + 1);
        key.push(tag);
        key.extend_from_slice(bytes);
        key
    }

    /// This value's components in `format`, or why it doesn't fit.
    fn convert(&self, format: VertexFormat) -> Result<Components, Misfit> {
        match *self {
            Self::F32(value) => convert_typed(format, VertexFormat::Float32, &[value.to_bits()]),
            Self::F32x2(values) => {
                convert_typed(format, VertexFormat::Float32x2, &values.map(f32::to_bits))
            }
            Self::F32x3(values) => {
                convert_typed(format, VertexFormat::Float32x3, &values.map(f32::to_bits))
            }
            Self::F32x4(values) => {
                convert_typed(format, VertexFormat::Float32x4, &values.map(f32::to_bits))
            }
            Self::U32(value) => convert_typed(format, VertexFormat::Uint32, &[value]),
            Self::U32x2(values) => convert_typed(format, VertexFormat::Uint32x2, &values),
            Self::U32x3(values) => convert_typed(format, VertexFormat::Uint32x3, &values),
            Self::U32x4(values) => convert_typed(format, VertexFormat::Uint32x4, &values),
            Self::Number(value) => convert_numbers(format, &[value]),
            Self::Number2(values) => convert_numbers(format, &values),
            Self::Number3(values) => convert_numbers(format, &values),
            Self::Number4(values) => convert_numbers(format, &values),
        }
    }
}

/// A typed value's components, when its own format is `format`.
fn convert_typed(
    format: VertexFormat,
    own: VertexFormat,
    bits: &[u32],
) -> Result<Components, Misfit> {
    if own != format {
        return Err(Misfit {
            expected: describe(format),
            found: describe(own).to_string(),
        });
    }
    Ok(Components::new(bits))
}

/// Plain numbers in `format`: floats as they are, integers when whole and in range.
fn convert_numbers(format: VertexFormat, numbers: &[f64]) -> Result<Components, Misfit> {
    let misfit = |found: String| Misfit {
        expected: describe(format),
        found,
    };
    let count = format!("{} number(s)", numbers.len());
    let Some((scalar, len)) = split(format) else {
        return Err(misfit(count));
    };
    if len != numbers.len() {
        return Err(misfit(count));
    }
    let unsigned = 0.0..=f64::from(u32::MAX);
    let signed = f64::from(i32::MIN)..=f64::from(i32::MAX);
    let mut bits = [0; 4];
    for (slot, &value) in bits.iter_mut().zip(numbers) {
        *slot = match scalar {
            Scalar::Float => (value as f32).to_bits(),
            Scalar::Uint if is_whole_in(value, &unsigned) => value as u32,
            Scalar::Sint if is_whole_in(value, &signed) => value as i32 as u32,
            _ => return Err(misfit(value.to_string())),
        };
    }
    Ok(Components { bits, len })
}

/// The scalar type and component count of a 32-bit format, the kind shader inputs reflect to.
fn split(format: VertexFormat) -> Option<(Scalar, usize)> {
    Some(match format {
        VertexFormat::Float32 => (Scalar::Float, 1),
        VertexFormat::Float32x2 => (Scalar::Float, 2),
        VertexFormat::Float32x3 => (Scalar::Float, 3),
        VertexFormat::Float32x4 => (Scalar::Float, 4),
        VertexFormat::Uint32 => (Scalar::Uint, 1),
        VertexFormat::Uint32x2 => (Scalar::Uint, 2),
        VertexFormat::Uint32x3 => (Scalar::Uint, 3),
        VertexFormat::Uint32x4 => (Scalar::Uint, 4),
        VertexFormat::Sint32 => (Scalar::Sint, 1),
        VertexFormat::Sint32x2 => (Scalar::Sint, 2),
        VertexFormat::Sint32x3 => (Scalar::Sint, 3),
        VertexFormat::Sint32x4 => (Scalar::Sint, 4),
        _ => return None,
    })
}

/// The WGSL type a format holds, such as `vec2<u32>`.
pub(crate) fn describe(format: VertexFormat) -> &'static str {
    match format {
        VertexFormat::Float32 => "f32",
        VertexFormat::Float32x2 => "vec2<f32>",
        VertexFormat::Float32x3 => "vec3<f32>",
        VertexFormat::Float32x4 => "vec4<f32>",
        VertexFormat::Uint32 => "u32",
        VertexFormat::Uint32x2 => "vec2<u32>",
        VertexFormat::Uint32x3 => "vec3<u32>",
        VertexFormat::Uint32x4 => "vec4<u32>",
        VertexFormat::Sint32 => "i32",
        VertexFormat::Sint32x2 => "vec2<i32>",
        VertexFormat::Sint32x3 => "vec3<i32>",
        VertexFormat::Sint32x4 => "vec4<i32>",
        _ => "an unsupported vertex format",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(value: VertexData, format: VertexFormat) -> Result<Vec<u8>, Misfit> {
        let mut out = Vec::new();
        value.write(format, &mut out).map(|()| out)
    }

    fn misfit(expected: &'static str, found: &str) -> Result<Vec<u8>, Misfit> {
        Err(Misfit {
            expected,
            found: found.to_string(),
        })
    }

    fn words(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect()
    }

    // Story: plain numbers take the declared type, exactly, at the edges of each range.
    #[test]
    fn numbers_take_the_declared_type() {
        let id = VertexData::Number(16_777_217.0);
        assert_eq!(encode(id, VertexFormat::Uint32), Ok(words(&[16_777_217])));
        assert_eq!(
            encode(
                VertexData::Number(f64::from(u32::MAX)),
                VertexFormat::Uint32
            ),
            Ok(words(&[u32::MAX]))
        );
        assert_eq!(
            encode(
                VertexData::Number2([f64::from(i32::MIN), 2.0]),
                VertexFormat::Sint32x2
            ),
            Ok(words(&[i32::MIN as u32, 2]))
        );
        assert_eq!(
            encode(
                VertexData::Number3([0.5, 1.0, -2.0]),
                VertexFormat::Float32x3
            ),
            Ok(words(&[
                0.5f32.to_bits(),
                1.0f32.to_bits(),
                (-2.0f32).to_bits()
            ]))
        );
    }

    // Story: a number an integer type can't hold, or the wrong count, names what it found.
    #[test]
    fn misfits_name_the_type_and_the_value() {
        assert_eq!(
            encode(VertexData::Number(-1.0), VertexFormat::Uint32),
            misfit("u32", "-1")
        );
        assert_eq!(
            encode(VertexData::Number2([1.0, 2.5]), VertexFormat::Sint32x2),
            misfit("vec2<i32>", "2.5")
        );
        assert_eq!(
            encode(VertexData::Number(f64::NAN), VertexFormat::Sint32),
            misfit("i32", "NaN")
        );
        assert_eq!(
            encode(VertexData::Number(4_294_967_296.0), VertexFormat::Uint32),
            misfit("u32", "4294967296")
        );
        assert_eq!(
            encode(VertexData::Number3([1.0, 2.0, 3.0]), VertexFormat::Sint32x2),
            misfit("vec2<i32>", "3 number(s)")
        );
    }

    // Story: an f32 or u32 from Rust keeps its type and only fits its own format.
    #[test]
    fn typed_values_must_match() {
        assert_eq!(
            encode(VertexData::U32(7), VertexFormat::Uint32),
            Ok(words(&[7]))
        );
        assert_eq!(
            encode(VertexData::U32(7), VertexFormat::Float32),
            misfit("f32", "u32")
        );
        assert_eq!(
            encode(VertexData::F32x2([0.5, 0.5]), VertexFormat::Sint32x2),
            misfit("vec2<i32>", "vec2<f32>")
        );
        assert_eq!(
            VertexData::F32x4([1.0; 4]).shape(),
            Shape::Typed(VertexFormat::Float32x4)
        );
        assert_eq!(
            VertexData::Number2([1.0; 2]).shape(),
            Shape::Numbers(Count::Two)
        );
    }

    // Story: a type a shader recorded wins, then a typed value's own format, then floats.
    #[test]
    fn resolves_the_recorded_type_first() {
        let recorded = Some(VertexFormat::Uint32);
        assert_eq!(VertexData::F32(1.0).resolve(recorded), VertexFormat::Uint32);
        assert_eq!(
            VertexData::Number(1.0).resolve(recorded),
            VertexFormat::Uint32
        );
        assert_eq!(VertexData::F32(1.0).resolve(None), VertexFormat::Float32);
        assert_eq!(
            VertexData::Number2([1.0; 2]).resolve(None),
            VertexFormat::Float32x2
        );
    }

    // Story: every binding sends 1 to 4 numbers, and any other count is an error.
    #[test]
    fn takes_one_to_four_numbers() {
        assert_eq!(
            VertexData::try_from(&[7.0][..]).ok(),
            Some(VertexData::Number(7.0))
        );
        assert_eq!(
            VertexData::try_from(&[1.0, 2.0, 3.0, 4.0][..]).ok(),
            Some(VertexData::Number4([1.0, 2.0, 3.0, 4.0]))
        );
        assert!(matches!(
            VertexData::try_from(&[][..]),
            Err(MeshError::Length(0))
        ));
        assert!(matches!(
            VertexData::try_from(&[1.0; 5][..]),
            Err(MeshError::Length(5))
        ));
    }

    // Story: two values that pack to different bytes never share a dedup key.
    #[test]
    fn keys_tell_types_apart() {
        let float = VertexData::F32x2([1.0, 0.0]);
        let unsigned = VertexData::U32x2([1.0f32.to_bits(), 0]);
        assert_ne!(float.to_key(), unsigned.to_key());
        assert_eq!(float.to_key(), VertexData::F32x2([1.0, 0.0]).to_key());
    }
}
