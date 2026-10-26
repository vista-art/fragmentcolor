#[cfg(python)]
pub mod python;

#[cfg(wasm)]
pub mod web;

/// A number from JavaScript or Python that doesn't fit an unsigned 32-bit integer.
#[cfg(any(wasm, python, test))]
#[derive(Debug, thiserror::Error)]
#[error("{what} must be a whole number from 0 to {}, got {value}", u32::MAX)]
pub(crate) struct NumberError {
    what: String,
    value: f64,
}

/// Reads a binding's number as an unsigned 32-bit integer, such as an index or a count.
/// Negative, fractional, NaN and out-of-range values fail with an error that names `what`.
#[cfg(any(wasm, python, test))]
pub(crate) fn read_unsigned(value: f64, what: &str) -> Result<u32, NumberError> {
    let whole = value.is_finite() && value.fract() == 0.0;
    if whole && (0.0..=u32::MAX as f64).contains(&value) {
        return Ok(value as u32);
    }
    Err(NumberError {
        what: what.to_string(),
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::read_unsigned;

    // Story: a count or an index takes a whole number from 0 to u32::MAX,
    // and anything else fails with a message that names the parameter.
    #[test]
    fn reads_whole_numbers_only() {
        assert_eq!(read_unsigned(0.0, "x").ok(), Some(0));
        assert_eq!(read_unsigned(4294967295.0, "x").ok(), Some(u32::MAX));
        for value in [-1.0, 0.5, 4294967296.0, f64::NAN, f64::INFINITY] {
            assert!(read_unsigned(value, "x").is_err(), "{value} should fail");
        }
        let message = read_unsigned(-1.0, "Pass.set_compute_dispatch: x")
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "Pass.set_compute_dispatch: x must be a whole number from 0 to 4294967295, got -1"
            )
        );
    }
}
