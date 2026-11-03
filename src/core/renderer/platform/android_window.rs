//! The size Android's `ANativeWindow` reports for a window target.
//! It uses no NDK types, so the tests run on the host.

use crate::InitializationError;

/// The window's width and height in pixels, each at least 1. Android reports a side
/// below 0 when `ANativeWindow_getWidth` or `ANativeWindow_getHeight` fails, which is an error.
pub(crate) fn read_size(width: i32, height: i32) -> Result<[u32; 2], InitializationError> {
    let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
        return Err(InitializationError::Error(format!(
            "createTarget: the ANativeWindow reports a size of {width} by {height}, and a side below 0 is an error"
        )));
    };
    Ok([width.max(1), height.max(1)])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: a side of 0 becomes 1 pixel, a positive size reads as it is,
    // and a side below 0 fails with an error that names the reported size.
    #[test]
    fn reads_window_sizes() {
        assert_eq!(read_size(0, 0).ok(), Some([1, 1]));
        assert_eq!(read_size(640, 480).ok(), Some([640, 480]));
        for (width, height) in [(-22, 100), (100, -22), (i32::MIN, 0)] {
            assert!(read_size(width, height).is_err(), "{width} by {height}");
        }
        let message = read_size(-22, 100).err().map(|error| error.to_string());
        assert_eq!(
            message.as_deref(),
            Some(
                "Initialization error: createTarget: the ANativeWindow reports a size of -22 by 100, and a side below 0 is an error"
            )
        );
    }
}
