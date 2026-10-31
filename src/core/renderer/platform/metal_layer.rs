//! Checks on the `CAMetalLayer` that iOS hands `Renderer::create_target_ios`.
//! They build on macOS too, where the tests run against real Objective-C objects.

use objc2::runtime::{AnyClass, AnyObject};

use crate::InitializationError;

/// Fails unless `object` is a `CAMetalLayer` or a subclass of one.
/// It reads the class chain and sends the object no message.
pub(crate) fn check_layer(object: &AnyObject) -> Result<(), InitializationError> {
    let Some(expected) = AnyClass::get(c"CAMetalLayer") else {
        return Err(InitializationError::Error(
            "createTarget: the CAMetalLayer class isn't loaded".into(),
        ));
    };
    let class = object.class();
    let mut current = Some(class);
    while let Some(candidate) = current {
        if candidate == expected {
            return Ok(());
        }
        current = candidate.superclass();
    }
    Err(InitializationError::Error(format!(
        "createTarget: expected a CAMetalLayer, got {}",
        class.name().to_string_lossy()
    )))
}

/// One side of the layer's drawable in pixels, at least 1.
/// The system reports it as a float, so a fraction rounds down.
/// A side that is negative, NaN or past `u32::MAX` fails.
pub(crate) fn read_side(pixels: f64) -> Result<u32, InitializationError> {
    if pixels.is_finite() && (0.0..=u32::MAX as f64).contains(&pixels) {
        return Ok((pixels as u32).max(1));
    }
    Err(InitializationError::Error(format!(
        "createTarget: the CAMetalLayer's drawableSize has a side of {pixels}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::msg_send;
    use objc2::rc::Retained;
    use std::ffi::CStr;

    fn create(name: &CStr) -> Retained<AnyObject> {
        let class = AnyClass::get(name).expect("the class is loaded");
        // SAFETY: `new` on an Objective-C class returns a retained instance.
        unsafe { msg_send![class, new] }
    }

    // Story: createTarget takes a CAMetalLayer and refuses any other object with an error.
    #[test]
    fn accepts_only_a_metal_layer() {
        assert!(check_layer(&create(c"CAMetalLayer")).is_ok());
        let error = check_layer(&create(c"CALayer"))
            .err()
            .map(|error| error.to_string());
        assert_eq!(
            error.as_deref(),
            Some("Initialization error: createTarget: expected a CAMetalLayer, got CALayer")
        );
        assert!(check_layer(&create(c"NSObject")).is_err());
    }

    // Story: a drawable side of zero becomes 1 pixel, and a negative or NaN one fails.
    #[test]
    fn reads_drawable_sides() {
        assert_eq!(read_side(0.0).ok(), Some(1));
        assert_eq!(read_side(640.5).ok(), Some(640));
        for pixels in [-1.0, f64::NAN, f64::INFINITY, 1e10] {
            assert!(read_side(pixels).is_err(), "{pixels} should fail");
        }
    }
}
