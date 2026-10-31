//! How every binding reads numbers:
//! finite 32-bit floats for colors, positions, directions, axes and UV transforms.

use crate::{Color, Vertex};

/// A float vector that can't be used, such as a color or a position.
#[derive(Debug, thiserror::Error)]
pub(crate) enum VectorError {
    #[error(
        "{label}: item {index} must be a finite number that fits a 32-bit float, got {}",
        show(*value)
    )]
    Item {
        label: String,
        index: usize,
        value: f64,
    },
    #[error("{label} takes {expected} numbers, got {count}")]
    Length {
        label: String,
        expected: String,
        count: usize,
    },
}

/// The number as an error prints it: a large one in exponent form, such as `1e39`.
fn show(value: f64) -> String {
    if value.is_finite() && value.abs() >= 1e16 {
        format!("{value:e}")
    } else {
        value.to_string()
    }
}

/// Reads each number of a float vector as an `f32`, the rule every binding shares.
/// NaN, an infinity, or a finite number past the `f32` range fails.
pub(crate) fn read_floats(
    numbers: impl IntoIterator<Item = impl Into<f64>>,
    label: &str,
) -> Result<Vec<f32>, VectorError> {
    numbers
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let value: f64 = value.into();
            let float = value as f32;
            if float.is_finite() {
                Ok(float)
            } else {
                Err(VectorError::Item {
                    label: label.to_string(),
                    index,
                    value,
                })
            }
        })
        .collect()
}

/// Exactly `N` numbers, read as [`read_floats`] reads them.
pub(crate) fn read_vector<const N: usize>(
    numbers: impl IntoIterator<Item = impl Into<f64>>,
    label: &str,
) -> Result<[f32; N], VectorError> {
    let floats = read_floats(numbers, label)?;
    match <[f32; N]>::try_from(floats.as_slice()) {
        Ok(vector) => Ok(vector),
        Err(_) => Err(VectorError::Length {
            label: label.to_string(),
            expected: N.to_string(),
            count: floats.len(),
        }),
    }
}

/// A color from `[r, g, b]` with an opaque alpha or `[r, g, b, a]`, read as [`read_floats`] reads them.
pub(crate) fn read_color(
    numbers: impl IntoIterator<Item = impl Into<f64>>,
    label: &str,
) -> Result<Color, VectorError> {
    let floats = read_floats(numbers, label)?;
    match Color::try_from(floats.as_slice()) {
        Ok(color) => Ok(color),
        Err(_) => Err(VectorError::Length {
            label: label.to_string(),
            expected: "3 or 4".to_string(),
            count: floats.len(),
        }),
    }
}

/// A vertex from a position of 1 to 4 numbers, read as [`read_floats`] reads them.
pub(crate) fn read_position(
    numbers: impl IntoIterator<Item = impl Into<f64>>,
    label: &str,
) -> Result<Vertex, VectorError> {
    match *read_floats(numbers, label)? {
        [x] => Ok(Vertex::new(x)),
        [x, y] => Ok(Vertex::new([x, y])),
        [x, y, z] => Ok(Vertex::new([x, y, z])),
        [x, y, z, w] => Ok(Vertex::new([x, y, z, w])),
        ref other => Err(VectorError::Length {
            label: label.to_string(),
            expected: "1 to 4".to_string(),
            count: other.len(),
        }),
    }
}

/// A float vector Python can't use raises `InvalidInputError`.
#[cfg(python)]
impl From<VectorError> for pyo3::PyErr {
    fn from(error: VectorError) -> Self {
        crate::error::raise_error(&error)
    }
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for VectorError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::InvalidInput
    }
}

/// A float vector Swift and Kotlin can't use throws a render error, as a bad region does.
#[cfg(mobile)]
impl From<VectorError> for crate::renderer::platform::mobile::FragmentColorError {
    fn from(error: VectorError) -> Self {
        Self::Render(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: a float vector takes finite numbers that fit an f32 and refuses
    // NaN, an infinity and a finite number past the f32 range, naming the item,
    // the rule JavaScript, Swift, Kotlin and Python share.
    #[test]
    fn reads_finite_floats_only() {
        assert_eq!(
            read_floats([0.5, -2.0, 3.4e38], "x").ok(),
            Some(vec![0.5, -2.0, 3.4e38])
        );
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e39, -1e39] {
            assert!(
                read_floats([0.0, value], "x").is_err(),
                "{value} should fail"
            );
        }
        let message = read_vector::<3>([0.0, 1e39, 0.0], "Light.point: position")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "Light.point: position: item 1 must be a finite number that fits a 32-bit float, got 1e39"
            )
        );
        let message = read_vector::<3>([0.0, 1.0], "Model.translate")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("Model.translate takes 3 numbers, got 2")
        );
        let color = read_color([0.5, 0.25, 1.0], "Pass.setClearColor").ok();
        assert_eq!(color, Some(Color::new(0.5, 0.25, 1.0, 1.0)));
        let message = read_color([f64::NAN, 0.0, 0.0], "Pass.setClearColor")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "Pass.setClearColor: item 0 must be a finite number that fits a 32-bit float, got NaN"
            )
        );
        let message = read_color([0.5; 5], "Pass.setClearColor")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("Pass.setClearColor takes 3 or 4 numbers, got 5")
        );
    }

    // Story: a vertex position takes 1 to 4 numbers under the same rule,
    // so a Vertex and a Quad corner refuse what a color refuses.
    #[test]
    fn reads_vertex_positions() {
        let fields = |vertex: Vertex| (vertex.position, vertex.dimensions);
        let vertex = read_position([0.5, -1.0, 2.0], "Vertex: position").ok();
        assert_eq!(
            vertex.map(fields),
            Some(fields(Vertex::new([0.5, -1.0, 2.0])))
        );
        let vertex = read_position([0.25_f32], "Vertex: position").ok();
        assert_eq!(vertex.map(fields), Some(fields(Vertex::new(0.25))));
        let message = read_position([0.0, f64::NAN], "Vertex: position")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "Vertex: position: item 1 must be a finite number that fits a 32-bit float, got NaN"
            )
        );
        let message = read_position([0.0; 5], "Vertex: position")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some("Vertex: position takes 1 to 4 numbers, got 5")
        );
        assert!(read_position(Vec::<f64>::new(), "Vertex: position").is_err());
    }
}
