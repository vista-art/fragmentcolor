//! `UniformInput`: a value as a binding
//! hands it over, shaped to the type the shader declares.

use super::UniformData;
use crate::ShaderError;
use crate::errors::show_number;
use crate::texture::{TextureId, TextureMeta};

/// A uniform value as a host language hands it over, before it has a WGSL type.
///
/// Each binding decodes its own values into this enum.
/// [`UniformInput::resolve`] shapes the value to the type declared for the key,
/// so every language accepts and rejects the same values under one set of rules.
/// Numbers travel as `f64` because it holds every `i32` and `u32` value exactly, unlike `f32`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UniformInput {
    Bool(bool),
    Number(f64),
    Numbers(Vec<f64>),
    Texture(TextureId),
    Bytes(Vec<u8>),
}

impl UniformInput {
    /// Shapes this value to `slot`, the type declared for `key`.
    pub(crate) fn resolve(self, key: &str, slot: &UniformData) -> Result<UniformData, ShaderError> {
        use UniformData as U;
        let numbers = match self {
            Self::Texture(id) => {
                return match slot {
                    U::Texture(_) => Ok(U::Texture(TextureMeta::from(id))),
                    _ => Err(mismatch(key, slot, "a texture")),
                };
            }
            Self::Bytes(bytes) => return fill_root(key, slot, bytes),
            Self::Bool(value) => vec![f64::from(u8::from(value))],
            Self::Number(value) => vec![value],
            Self::Numbers(values) => values,
        };
        match slot {
            U::Bool(_) => floats(key, slot, &numbers, 1).map(|v| U::Bool(v[0] != 0.0)),
            U::Float(_) => floats(key, slot, &numbers, 1).map(|v| U::Float(v[0])),
            U::Int(_) => ints(key, slot, &numbers, 1).map(|v| U::Int(v[0])),
            U::UInt(_) => uints(key, slot, &numbers, 1).map(|v| U::UInt(v[0])),
            U::Vec2(_) => floats(key, slot, &numbers, 2).map(U::Vec2),
            U::Vec3(_) => floats(key, slot, &numbers, 3).map(U::Vec3),
            U::Vec4(_) => floats(key, slot, &numbers, 4).map(U::Vec4),
            U::IVec2(_) => ints(key, slot, &numbers, 2).map(U::IVec2),
            U::IVec3(_) => ints(key, slot, &numbers, 3).map(U::IVec3),
            U::IVec4(_) => ints(key, slot, &numbers, 4).map(U::IVec4),
            U::UVec2(_) => uints(key, slot, &numbers, 2).map(U::UVec2),
            U::UVec3(_) => uints(key, slot, &numbers, 3).map(U::UVec3),
            U::UVec4(_) => uints(key, slot, &numbers, 4).map(U::UVec4),
            U::Mat2(_) => floats(key, slot, &numbers, 4).map(U::Mat2),
            U::Mat3(_) => floats(key, slot, &numbers, 9).map(U::Mat3),
            U::Mat4(_) => floats(key, slot, &numbers, 16).map(U::Mat4),
            U::Half(_) => round_halves(key, slot, &numbers, 1).map(|v| U::Half(v[0])),
            U::HalfVec2(_) => round_halves(key, slot, &numbers, 2).map(U::HalfVec2),
            U::HalfVec3(_) => round_halves(key, slot, &numbers, 3).map(U::HalfVec3),
            U::HalfVec4(_) => round_halves(key, slot, &numbers, 4).map(U::HalfVec4),
            U::HalfMat2(_) => round_halves(key, slot, &numbers, 4).map(U::HalfMat2),
            U::HalfMat3(_) => round_halves(key, slot, &numbers, 9).map(U::HalfMat3),
            U::HalfMat4(_) => round_halves(key, slot, &numbers, 16).map(U::HalfMat4),
            _ => Err(count_mismatch(key, slot, &numbers)),
        }
    }
}

/// Lets typed Rust values take the same path as the bindings.
/// Shapes with no loose form, such as a struct or a sampler, come back as the error.
impl TryFrom<UniformData> for UniformInput {
    type Error = UniformData;

    fn try_from(data: UniformData) -> Result<Self, Self::Error> {
        use UniformData as U;
        fn widen<T: Into<f64>>(values: Vec<T>) -> UniformInput {
            UniformInput::Numbers(values.into_iter().map(Into::into).collect())
        }
        Ok(match data {
            U::Bool(value) => Self::Bool(value),
            U::Float(value) => Self::Number(value.into()),
            U::Int(value) => Self::Number(value.into()),
            U::UInt(value) => Self::Number(value.into()),
            U::Vec2(v) | U::Vec3(v) | U::Vec4(v) | U::Mat2(v) | U::Mat3(v) | U::Mat4(v) => widen(v),
            U::Half(value) => Self::Number(value.into()),
            U::HalfVec2(v) | U::HalfVec3(v) | U::HalfVec4(v) => widen(v),
            U::HalfMat2(v) | U::HalfMat3(v) | U::HalfMat4(v) => widen(v),
            U::IVec2(v) | U::IVec3(v) | U::IVec4(v) => widen(v),
            U::UVec2(v) | U::UVec3(v) | U::UVec4(v) => widen(v),
            U::Texture(meta) => Self::Texture(meta.id),
            U::Bytes(bytes) => Self::Bytes(bytes),
            other => return Err(other),
        })
    }
}

/// Raw bytes for a storage or push-constant root, which must be exactly its size.
fn fill_root(key: &str, slot: &UniformData, bytes: Vec<u8>) -> Result<UniformData, ShaderError> {
    let span = match slot {
        UniformData::Storage(entries) => entries.first().map(|entry| entry.span),
        UniformData::PushConstant(entries) => entries.first().map(|entry| entry.span),
        _ => None,
    };
    match span {
        Some(span) if bytes.len() == span as usize => Ok(UniformData::Bytes(bytes)),
        Some(span) => Err(refuse_count(key, span as usize, bytes.len())),
        None => Err(mismatch(key, slot, "raw bytes")),
    }
}

/// Builds an error like `particles: expected 8192 bytes, got 3`.
pub(crate) fn refuse_count(key: &str, span: usize, count: usize) -> ShaderError {
    ShaderError::TypeMismatch(format!("{key}: expected {span} bytes, got {count}"))
}

/// Refuses a value with no loose form, which `set` takes in parts or as bytes,
/// with an error like `u: expected numbers, a texture or bytes, got a whole struct`.
pub(crate) fn refuse_shape(key: &str, shape: &UniformData) -> ShaderError {
    use UniformData as U;
    let found = match shape {
        U::Struct(_) => "a whole struct (set one field with key.field)",
        U::Array(_) => "a whole array (set one element with key[i])",
        U::Storage(_) | U::PushConstant(_) => "a buffer root's shape (set its bytes instead)",
        U::Sampler(_) => "a sampler, which binds automatically",
        _ => "a value with no loose form",
    };
    ShaderError::TypeMismatch(format!(
        "{key}: expected numbers, a texture or bytes, got {found}"
    ))
}

/// Rounds `n` numbers to the nearest `f32`, refusing NaN, an infinity or a number past its range.
fn floats(
    key: &str,
    slot: &UniformData,
    numbers: &[f64],
    n: usize,
) -> Result<Vec<f32>, ShaderError> {
    round_floats(key, slot, numbers, n, |value| value as f32)
}

/// Rounds `n` numbers to the nearest `f16`, refusing NaN, an infinity,
/// or a number past ±65504 that doesn't round back to it.
fn round_halves(
    key: &str,
    slot: &UniformData,
    numbers: &[f64],
    n: usize,
) -> Result<Vec<f32>, ShaderError> {
    round_floats(key, slot, numbers, n, |value| {
        half::f16::from_f64(value).to_f32()
    })
}

/// Rounds `n` numbers with `round`, refusing one that isn't finite after rounding,
/// with an error that names the number as given. Nothing is clamped.
fn round_floats(
    key: &str,
    slot: &UniformData,
    numbers: &[f64],
    n: usize,
    round: fn(f64) -> f32,
) -> Result<Vec<f32>, ShaderError> {
    if numbers.len() != n {
        return Err(count_mismatch(key, slot, numbers));
    }
    let rounded = numbers.iter().map(|&value| (value, round(value)));
    rounded
        .map(|(value, float)| match float.is_finite() {
            true => Ok(float),
            false => Err(mismatch(key, slot, &show_number(value))),
        })
        .collect()
}

fn ints(key: &str, slot: &UniformData, numbers: &[f64], n: usize) -> Result<Vec<i32>, ShaderError> {
    let range = f64::from(i32::MIN)..=f64::from(i32::MAX);
    whole(key, slot, numbers, n, range).map(|v| v.into_iter().map(|v| v as i32).collect())
}

fn uints(
    key: &str,
    slot: &UniformData,
    numbers: &[f64],
    n: usize,
) -> Result<Vec<u32>, ShaderError> {
    let range = 0.0..=f64::from(u32::MAX);
    whole(key, slot, numbers, n, range).map(|v| v.into_iter().map(|v| v as u32).collect())
}

/// Checks that there are `n` numbers and that each is a whole number inside `range`.
fn whole(
    key: &str,
    slot: &UniformData,
    numbers: &[f64],
    n: usize,
    range: std::ops::RangeInclusive<f64>,
) -> Result<Vec<f64>, ShaderError> {
    if numbers.len() != n {
        return Err(count_mismatch(key, slot, numbers));
    }
    match numbers.iter().find(|&&v| !is_whole_in(v, &range)) {
        Some(bad) => Err(mismatch(key, slot, &bad.to_string())),
        None => Ok(numbers.to_vec()),
    }
}

/// Whether `value` is a whole number inside `range`. NaN and infinities never are.
pub(crate) fn is_whole_in(value: f64, range: &std::ops::RangeInclusive<f64>) -> bool {
    value.fract() == 0.0 && range.contains(&value)
}

fn count_mismatch(key: &str, slot: &UniformData, numbers: &[f64]) -> ShaderError {
    mismatch(key, slot, &format!("{} number(s)", numbers.len()))
}

/// Builds an error like `resolution: expected vec2<f32>, got 3 number(s)`.
fn mismatch(key: &str, slot: &UniformData, found: &str) -> ShaderError {
    use UniformData as U;
    let expected = match slot {
        U::Bool(_) => "bool",
        U::Float(_) => "f32",
        U::Int(_) => "i32",
        U::UInt(_) => "u32",
        U::Vec2(_) => "vec2<f32>",
        U::Vec3(_) => "vec3<f32>",
        U::Vec4(_) => "vec4<f32>",
        U::IVec2(_) => "vec2<i32>",
        U::IVec3(_) => "vec3<i32>",
        U::IVec4(_) => "vec4<i32>",
        U::UVec2(_) => "vec2<u32>",
        U::UVec3(_) => "vec3<u32>",
        U::UVec4(_) => "vec4<u32>",
        U::Mat2(_) => "mat2x2<f32>",
        U::Mat3(_) => "mat3x3<f32>",
        U::Mat4(_) => "mat4x4<f32>",
        U::Half(_) => "f16",
        U::HalfVec2(_) => "vec2<f16>",
        U::HalfVec3(_) => "vec3<f16>",
        U::HalfVec4(_) => "vec4<f16>",
        U::HalfMat2(_) => "mat2x2<f16>",
        U::HalfMat3(_) => "mat3x3<f16>",
        U::HalfMat4(_) => "mat4x4<f16>",
        U::Texture(_) => "a texture",
        U::Sampler(_) => "a sampler, which binds automatically",
        U::Array(_) => "an array (set one element with key[i])",
        U::Struct(_) => "a struct (set one field with key.field)",
        U::Storage(_) | U::PushConstant(_) => "raw bytes (or set one field with key.field)",
        U::Bytes(_) => "raw bytes",
    };
    ShaderError::TypeMismatch(format!("{key}: expected {expected}, got {found}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PushEntry, StorageAccess, StorageEntry, StructShape};

    fn resolve(input: UniformInput, slot: UniformData) -> Result<UniformData, String> {
        input.resolve("key", &slot).map_err(|e| e.to_string())
    }

    fn texture_slot() -> UniformData {
        UniformData::Texture(TextureMeta::from(TextureId { id: 0 }))
    }

    // Story: one number fits an f32, i32 or u32 slot, and the declared type decides.
    #[test]
    fn numbers_take_the_declared_scalar_type() {
        let three = || UniformInput::Number(3.0);
        assert_eq!(
            resolve(three(), UniformData::Float(0.0)),
            Ok(UniformData::Float(3.0))
        );
        assert_eq!(
            resolve(three(), UniformData::Int(0)),
            Ok(UniformData::Int(3))
        );
        assert_eq!(
            resolve(three(), UniformData::UInt(0)),
            Ok(UniformData::UInt(3))
        );
    }

    // Story: integer slots take whole numbers inside their range, exactly.
    #[test]
    fn integer_slots_take_whole_numbers_in_range() {
        let uint = UniformData::UInt(0);
        let int = UniformData::Int(0);
        let max = UniformInput::Number(f64::from(u32::MAX));
        assert_eq!(resolve(max, uint.clone()), Ok(UniformData::UInt(u32::MAX)));
        // 2^24 + 1 has no exact f32, so a float transport would corrupt it.
        let seed = UniformInput::Number(16_777_217.0);
        assert_eq!(
            resolve(seed, uint.clone()),
            Ok(UniformData::UInt(16_777_217))
        );
        let negative = UniformInput::Number(-1.0);
        assert_eq!(
            resolve(negative.clone(), int.clone()),
            Ok(UniformData::Int(-1))
        );

        let err = resolve(negative, uint.clone()).unwrap_err();
        assert!(err.ends_with("key: expected u32, got -1"), "{err}");
        let err = resolve(UniformInput::Number(3.5), int.clone()).unwrap_err();
        assert!(err.ends_with("key: expected i32, got 3.5"), "{err}");
        let too_big = UniformInput::Number(f64::from(i32::MAX) + 1.0);
        assert!(resolve(too_big, int).is_err());
        assert!(resolve(UniformInput::Number(f64::NAN), uint).is_err());
    }

    // Story: vectors need exactly as many numbers as the WGSL type has components.
    #[test]
    fn vectors_need_exact_lengths() {
        let pair = UniformInput::Numbers(vec![1.0, 2.0]);
        let slot = UniformData::UVec2(vec![0; 2]);
        assert_eq!(
            resolve(pair, slot.clone()),
            Ok(UniformData::UVec2(vec![1, 2]))
        );

        let three = UniformInput::Numbers(vec![1.0, 2.0, 3.0]);
        let err = resolve(three.clone(), slot).unwrap_err();
        assert!(
            err.ends_with("key: expected vec2<u32>, got 3 number(s)"),
            "{err}"
        );
        let err = resolve(three, UniformData::Vec4(vec![0.0; 4])).unwrap_err();
        assert!(
            err.ends_with("key: expected vec4<f32>, got 3 number(s)"),
            "{err}"
        );
    }

    // Story: matrices take 4, 9 or 16 numbers, so a mat2x2 is reachable from any list.
    #[test]
    fn matrices_take_four_nine_or_sixteen_numbers() {
        let numbers = |n: usize| UniformInput::Numbers((0..n).map(|v| v as f64).collect());
        let mat2 = resolve(numbers(4), UniformData::Mat2(vec![0.0; 4]));
        assert_eq!(mat2, Ok(UniformData::Mat2(vec![0.0, 1.0, 2.0, 3.0])));
        assert!(resolve(numbers(9), UniformData::Mat3(vec![0.0; 9])).is_ok());
        assert!(resolve(numbers(16), UniformData::Mat4(vec![0.0; 16])).is_ok());
        let err = resolve(numbers(9), UniformData::Mat2(vec![0.0; 4])).unwrap_err();
        assert!(
            err.ends_with("key: expected mat2x2<f32>, got 9 number(s)"),
            "{err}"
        );
    }

    // Story: a bool writes 1 or 0 into a numeric slot.
    #[test]
    fn bools_write_one_or_zero() {
        let on = UniformInput::Bool(true);
        assert_eq!(
            resolve(on.clone(), UniformData::UInt(0)),
            Ok(UniformData::UInt(1))
        );
        assert_eq!(
            resolve(on, UniformData::Float(0.0)),
            Ok(UniformData::Float(1.0))
        );
        let off = UniformInput::Bool(false);
        assert_eq!(resolve(off, UniformData::Int(7)), Ok(UniformData::Int(0)));
    }

    // Story: a texture binds only to a texture slot, and numbers never do.
    #[test]
    fn textures_bind_only_to_texture_slots() {
        let texture = UniformInput::Texture(TextureId { id: 7 });
        let bound = resolve(texture.clone(), texture_slot());
        let expected = UniformData::Texture(TextureMeta::from(TextureId { id: 7 }));
        assert_eq!(bound, Ok(expected));

        let err = resolve(texture, UniformData::Vec2(vec![0.0; 2])).unwrap_err();
        assert!(
            err.ends_with("key: expected vec2<f32>, got a texture"),
            "{err}"
        );
        let err = resolve(UniformInput::Number(1.0), texture_slot()).unwrap_err();
        assert!(
            err.ends_with("key: expected a texture, got 1 number(s)"),
            "{err}"
        );
    }

    // Story: raw bytes fill storage and push-constant roots, and nothing else.
    #[test]
    fn bytes_fill_only_buffer_roots() {
        let bytes = UniformInput::Bytes(vec![1, 2, 3, 4]);
        let storage = UniformData::Storage(vec![StorageEntry {
            inner: vec![UniformData::Float(0.0)],
            span: 4,
            access: StorageAccess::Read,
        }]);
        let push = UniformData::PushConstant(vec![PushEntry {
            inner: vec![UniformData::Float(0.0)],
            span: 4,
        }]);
        let expected = Ok(UniformData::Bytes(vec![1, 2, 3, 4]));
        assert_eq!(resolve(bytes.clone(), storage), expected);
        assert_eq!(resolve(bytes.clone(), push), expected);

        let err = resolve(bytes.clone(), UniformData::Vec4(vec![0.0; 4])).unwrap_err();
        assert!(
            err.ends_with("key: expected vec4<f32>, got raw bytes"),
            "{err}"
        );
        let shape = StructShape {
            fields: Vec::new(),
            size: 4,
        };
        assert!(resolve(bytes, UniformData::Struct(shape)).is_err());
    }

    // Story: raw bytes replace a whole root, so too few and too many both fail.
    #[test]
    fn bytes_must_fill_the_whole_root() {
        let storage = UniformData::Storage(vec![StorageEntry {
            inner: vec![UniformData::Vec4(vec![0.0; 4])],
            span: 16,
            access: StorageAccess::Read,
        }]);
        let push = UniformData::PushConstant(vec![PushEntry {
            inner: vec![UniformData::Vec2(vec![0.0; 2])],
            span: 8,
        }]);
        let short = UniformInput::Bytes(vec![0; 3]);
        let err = resolve(short.clone(), storage.clone()).unwrap_err();
        assert!(err.ends_with("key: expected 16 bytes, got 3"), "{err}");
        let long = UniformInput::Bytes(vec![0; 20]);
        let err = resolve(long.clone(), storage).unwrap_err();
        assert!(err.ends_with("key: expected 16 bytes, got 20"), "{err}");
        let err = resolve(short, push.clone()).unwrap_err();
        assert!(err.ends_with("key: expected 8 bytes, got 3"), "{err}");
        assert!(resolve(long, push).is_err());
    }

    // Story: an f32 slot, a vector or matrix one included, takes any number that is finite
    // after rounding to f32 and refuses NaN, infinities and numbers past the f32 range,
    // naming the key, the type and the value, as an f16 slot does.
    #[test]
    fn float_slots_refuse_values_past_f32() {
        let float = UniformData::Float(0.0);
        let take = |value: f64| resolve(UniformInput::Number(value), float.clone());
        assert_eq!(take(0.1), Ok(UniformData::Float(0.1)));
        assert_eq!(take(-3.4e38), Ok(UniformData::Float(-3.4e38)));
        assert_eq!(take(1e-50), Ok(UniformData::Float(0.0)));
        for (value, shown) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
            (1e39, "1e39"),
            (-1e39, "-1e39"),
        ] {
            let err = take(value).unwrap_err();
            assert!(
                err.ends_with(&format!("key: expected f32, got {shown}")),
                "{err}"
            );
        }
        let numbers = UniformInput::Numbers(vec![0.0, 1.0, f64::NAN, 0.0]);
        let err = resolve(numbers, UniformData::Mat2(vec![0.0; 4])).unwrap_err();
        assert!(err.ends_with("key: expected mat2x2<f32>, got NaN"), "{err}");
        let err = resolve(UniformInput::Number(f64::NAN), UniformData::Bool(false)).unwrap_err();
        assert!(err.ends_with("key: expected bool, got NaN"), "{err}");
        // A Float value built in Rust takes the same path.
        let input = UniformInput::try_from(UniformData::Float(f32::INFINITY)).expect("number");
        assert!(resolve(input, float).is_err());
    }

    // Story: an f16 slot rounds each number to the nearest f16 and refuses one that isn't
    // finite after rounding, naming the key, the type and the value. Nothing is clamped.
    #[test]
    fn half_slots_round_and_refuse_values_past_f16() {
        let half = UniformData::Half(0.0);
        let take = |value: f64| resolve(UniformInput::Number(value), half.clone());
        assert_eq!(take(0.1), Ok(UniformData::Half(0.099_975_586)));
        assert_eq!(take(65504.0), Ok(UniformData::Half(65504.0)));
        assert_eq!(take(65519.0), Ok(UniformData::Half(65504.0)));
        assert_eq!(take(-65519.0), Ok(UniformData::Half(-65504.0)));
        assert_eq!(take(1e-9), Ok(UniformData::Half(0.0)));
        for (value, shown) in [
            (65520.0, "65520"),
            (70000.0, "70000"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
            (1e39, "1e39"),
        ] {
            let err = take(value).unwrap_err();
            assert!(
                err.ends_with(&format!("key: expected f16, got {shown}")),
                "{err}"
            );
        }

        let three = UniformInput::Numbers(vec![1.0, 2.0, 70000.0]);
        let err = resolve(three, UniformData::HalfVec3(vec![0.0; 3])).unwrap_err();
        assert!(err.ends_with("key: expected vec3<f16>, got 70000"), "{err}");
        let two = UniformInput::Numbers(vec![1.0, 2.0]);
        let err = resolve(two, UniformData::HalfVec3(vec![0.0; 3])).unwrap_err();
        assert!(
            err.ends_with("key: expected vec3<f16>, got 2 number(s)"),
            "{err}"
        );
        let numbers = |n: usize| UniformInput::Numbers((0..n).map(|v| v as f64).collect());
        let shapes = [
            (UniformData::HalfVec2(vec![0.0; 2]), 2),
            (UniformData::HalfVec4(vec![0.0; 4]), 4),
            (UniformData::HalfMat2(vec![0.0; 4]), 4),
            (UniformData::HalfMat3(vec![0.0; 9]), 9),
            (UniformData::HalfMat4(vec![0.0; 16]), 16),
        ];
        for (slot, n) in shapes {
            let resolved = resolve(numbers(n), slot.clone()).expect("numbers");
            assert_eq!(
                std::mem::discriminant(&resolved),
                std::mem::discriminant(&slot)
            );
            assert!(resolve(numbers(n + 1), slot).is_err());
        }
        assert_eq!(
            resolve(UniformInput::Bool(true), half.clone()),
            Ok(UniformData::Half(1.0))
        );

        // A Half value built in Rust takes the same path, so it is checked too.
        let input = UniformInput::try_from(UniformData::Half(70000.0)).expect("number");
        assert_eq!(input, UniformInput::Number(70000.0));
        assert!(resolve(input, half).is_err());
        let input = UniformInput::try_from(UniformData::HalfMat2(vec![1.0; 4]));
        assert_eq!(input, Ok(UniformInput::Numbers(vec![1.0; 4])));
    }

    // Story: typed Rust values flatten into the same loose input the bindings send.
    #[test]
    fn typed_rust_values_flatten_to_inputs() {
        let input = UniformInput::try_from(UniformData::IVec2(vec![1, -2]));
        assert_eq!(input, Ok(UniformInput::Numbers(vec![1.0, -2.0])));
        let input = UniformInput::try_from(UniformData::UInt(u32::MAX));
        assert_eq!(input, Ok(UniformInput::Number(f64::from(u32::MAX))));
        let shape = UniformData::Struct(StructShape {
            fields: Vec::new(),
            size: 0,
        });
        assert_eq!(UniformInput::try_from(shape.clone()), Err(shape));
    }
}
