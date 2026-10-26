//! GPU errors that a wasm frame reports after `render` has returned.

use super::RendererError;
use crate::texture::TextureError;
use parking_lot::Mutex;

/// A validation error that an error scope caught after its call returned.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LateError {
    pub(crate) label: String,
    pub(crate) message: String,
}

impl From<LateError> for RendererError {
    fn from(error: LateError) -> Self {
        RendererError::ValidationError {
            label: error.label,
            message: error.message,
        }
    }
}

impl From<LateError> for TextureError {
    fn from(error: LateError) -> Self {
        TextureError::Error(RendererError::from(error).to_string())
    }
}

/// Holds the first late error until the next call that touches the GPU takes it.
#[derive(Debug, Default)]
pub(crate) struct ErrorSlot(Mutex<Option<LateError>>);

impl ErrorSlot {
    /// Keeps `error`, unless an earlier one is still waiting.
    pub(crate) fn store(&self, error: LateError) {
        self.0.lock().get_or_insert(error);
    }

    /// Returns the waiting error, once, and empties the slot.
    pub(crate) fn take(&self) -> Result<(), LateError> {
        match self.0.lock().take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Stores the error that `popped` resolves to, when it resolves.
    ///
    /// A call that `failure` says already failed has returned its own error,
    /// so a GPU error caught alongside it goes to the log as a warning. The
    /// browser turns `log::error!` into a `console.error`, which the web healthcheck fails on.
    #[cfg(wasm)]
    pub(crate) fn watch(
        self: &std::sync::Arc<Self>,
        label: &str,
        popped: impl std::future::Future<Output = Option<wgpu::Error>> + 'static,
        failure: Option<String>,
    ) {
        let slot = self.clone();
        let label = label.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let Some(gpu) = popped.await else {
                return;
            };
            match failure {
                Some(failure) => {
                    log::warn!("{label} failed: {failure}. The GPU also reported: {gpu}")
                }
                None => slot.store(LateError {
                    label,
                    message: gpu.to_string(),
                }),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn late(label: &str) -> LateError {
        LateError {
            label: label.into(),
            message: format!("{label} failed"),
        }
    }

    // Story: the first error to land is the one the next call reports.
    #[test]
    fn keeps_the_first_error() {
        // Arrange
        let slot = ErrorSlot::default();

        // Act
        slot.store(late("render pipeline"));
        slot.store(late("frame"));

        // Assert
        assert_eq!(slot.take(), Err(late("render pipeline")));
    }

    // Story: an error is reported once, and the slot takes a new one afterwards.
    #[test]
    fn reports_each_error_once() {
        // Arrange
        let slot = ErrorSlot::default();
        slot.store(late("frame"));

        // Act
        let first = slot.take();
        let second = slot.take();
        slot.store(late("compute pipeline"));
        let third = slot.take();

        // Assert
        assert_eq!(first, Err(late("frame")));
        assert_eq!(second, Ok(()));
        assert_eq!(third, Err(late("compute pipeline")));
    }

    // Story: render reports a late error as a ValidationError, and a readback
    // as a TextureError that carries the same message.
    #[test]
    fn converts_to_each_call_error() {
        // Arrange
        let error = late("frame");

        // Act
        let renderer = RendererError::from(error.clone());
        let texture = TextureError::from(error);

        // Assert
        assert!(
            matches!(&renderer, RendererError::ValidationError { label, .. } if label == "frame"),
            "got {renderer:?}"
        );
        assert_eq!(
            texture.to_string(),
            format!("Renderer error: {renderer}"),
            "the readback error carries the render error's message"
        );
    }
}
