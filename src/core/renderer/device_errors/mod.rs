//! Runs GPU work in the error scopes that catch its `DeviceError`,
//! and holds `Start`, where a call that waits for earlier frames begins.

#[cfg(any(wasm, test))]
pub(super) mod late;
pub(super) mod scopes;

use crate::core::renderer::{RenderContext, RendererError};
#[cfg(not(wasm))]
use scopes::Origin;
use scopes::{Caught, SCOPES};

/// When a call that waits for earlier frames started. On the web it holds the
/// ticket of the context's error slot at that moment, so the call takes only
/// the error of a frame rendered before it, and a later frame keeps its error for the next call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Start {
    #[cfg(wasm)]
    before: u64,
}

impl Start {
    /// A call that starts now on a renderer with `context`, or with none yet,
    /// which leaves no earlier frame to wait for.
    pub(crate) fn now(context: Option<&RenderContext>) -> Self {
        #[cfg(wasm)]
        {
            Start {
                before: context.map_or(0, |context| context.late.mark()),
            }
        }
        #[cfg(not(wasm))]
        {
            let _ = context;
            Start {}
        }
    }

    /// Waits for the frames rendered before the start, then returns the first error they left.
    #[cfg(wasm)]
    pub(crate) async fn settle(self, context: &RenderContext) -> Result<(), late::LateError> {
        context.late.settle_from(self.before).await
    }
}

impl RenderContext {
    /// Runs `op` like `validate_with`, with nothing to run for a late error.
    /// Only native code calls it, since the wasm callers each pass `on_error`.
    #[cfg(not(wasm))]
    pub(super) fn validate<T>(
        &self,
        label: &str,
        op: impl FnOnce() -> Result<T, RendererError>,
    ) -> Result<T, RendererError> {
        self.validate_with(label, op, || {})
    }

    /// Runs `op` inside error scopes for out of memory, internal and validation errors.
    ///
    /// A wgpu error that `op` causes returns as `DeviceError`, with the kind of its scope,
    /// instead of reaching the device's uncaptured error handler. Pipeline builds and each frame
    /// go through here, and on native so does each bind group and view inside a frame. The scopes pop on every path.
    /// Of the errors they catch, the cause returns: out of memory, then internal, then validation.
    ///
    /// On native the scopes resolve before this returns. When `op` fails with a GPU error
    /// of its own, from a nested scope or a surface that failed validation, the cause these
    /// scopes caught came first, and it returns under `label`. When `op` fails on the CPU, its error
    /// returns. Every error that doesn't return goes to the log as a warning (`scopes::resolve`).
    ///
    /// On wasm the scopes resolve after `render` returns, so `op`'s own result always returns.
    /// `ErrorSlot::watch` logs each error that lands at error level, and keeps the cause in `late`
    /// for the next call only when `op` succeeded. When `op` failed, the cause is logged and dropped.
    /// In both cases `on_error` runs when the error lands. On native it never runs, since the error returns.
    pub(super) fn validate_with<T>(
        &self,
        label: &str,
        op: impl FnOnce() -> Result<T, RendererError>,
        on_error: impl FnOnce() + 'static,
    ) -> Result<T, RendererError> {
        let [memory, internal, validation] =
            SCOPES.map(|(filter, _)| self.device.push_error_scope(filter));
        #[cfg(not(wasm))]
        Origin::Cpu.record();
        let result = op();
        #[cfg(not(wasm))]
        let origin = Origin::take();
        // Scopes pop in reverse order.
        let validation = validation.pop();
        let internal = internal.pop();
        let memory = memory.pop();
        let caught = async move { Caught::split([memory.await, internal.await, validation.await]) };
        #[cfg(not(wasm))]
        {
            let _ = on_error;
            let caught = futures::executor::block_on(caught);
            let (result, origin) = scopes::resolve(label, result, origin, caught);
            origin.record();
            result
        }
        #[cfg(wasm)]
        {
            let failure = result.as_ref().err().map(ToString::to_string);
            self.late.watch(label, caught, failure, on_error);
            result
        }
    }

    /// Runs one step inside a frame, such as a bind group, in scopes of its own on native,
    /// so its error carries the step's label. On wasm the frame's scopes cover it.
    pub(super) fn validate_step<T>(
        &self,
        label: &str,
        op: impl FnOnce() -> Result<T, RendererError>,
    ) -> Result<T, RendererError> {
        #[cfg(not(wasm))]
        {
            self.validate(label, op)
        }
        #[cfg(wasm)]
        {
            let _ = label;
            op()
        }
    }
}
