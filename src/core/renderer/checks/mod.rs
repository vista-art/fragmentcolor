//! The CPU checks a render runs before any wgpu call, and `check_limit`.

use crate::core::renderer::RendererError;

pub(super) mod pass;
pub(super) mod shader;
mod texture_conflicts;

/// Refuses `item`, which needs `value` of the device's `limit`, when the device has less.
pub(crate) fn check_limit(
    item: &str,
    limit: &str,
    value: u64,
    max: u64,
) -> Result<(), RendererError> {
    match value > max {
        true => Err(RendererError::LimitError {
            item: item.to_string(),
            limit: limit.to_string(),
            value,
            max,
        }),
        false => Ok(()),
    }
}
