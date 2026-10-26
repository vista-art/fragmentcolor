//! The checks a render runs on the CPU before it records a pass.
//!
//! Each rule here is one wgpu enforces too. Checking it first returns a typed
//! error from the call that broke it, on the web as well, where WebGPU reports
//! its own errors only after `render` has returned.

use super::{RenderContext, RendererError};
use crate::Target;
use std::sync::Arc;

impl RenderContext {
    /// Refuses a target another Renderer created, since its frames belong to another device.
    pub(super) fn check_owner(&self, target: &impl Target) -> Result<(), RendererError> {
        match target.owner() {
            Some(owner) if !std::ptr::eq(Arc::as_ptr(&owner), self) => {
                Err(RendererError::ForeignTarget)
            }
            _ => Ok(()),
        }
    }
}
