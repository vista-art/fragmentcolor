//! GPU errors that a wasm frame reports after `render` has returned.

use super::RendererError;
#[cfg(wasm)]
use super::scope::Caught;
#[cfg(wasm)]
use crate::platforms::web::yield_to_event_loop;
use crate::texture::TextureError;
use parking_lot::Mutex;
use std::collections::BTreeSet;
#[cfg(wasm)]
use std::future::Future;
#[cfg(wasm)]
use std::sync::Arc;

/// A GPU error that an error scope caught after its call returned.
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
        TextureError::ValidationError {
            label: error.label,
            message: error.message,
        }
    }
}

/// Holds the first late error until the next call that reports it takes it.
#[derive(Debug, Default)]
pub(crate) struct ErrorSlot {
    first: Mutex<Option<LateError>>,
    scopes: Mutex<OpenScopes>,
}

/// The scopes whose errors haven't landed yet, by the ticket each one opened with.
#[derive(Debug, Default)]
struct OpenScopes {
    next: u64,
    open: BTreeSet<u64>,
}

/// Turns of the event loop `settle` waits for open scopes, about ten seconds.
#[cfg(wasm)]
const SETTLE_ROUNDS: usize = 2000;

impl ErrorSlot {
    /// Keeps `error`, unless an earlier one is still waiting.
    pub(crate) fn store(&self, error: LateError) {
        self.first.lock().get_or_insert(error);
    }

    /// Returns the waiting error, once, and empties the slot.
    pub(crate) fn take(&self) -> Result<(), LateError> {
        match self.first.lock().take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Waits until every scope opened before this call has landed,
    /// then returns the waiting error, once, like `take`.
    #[cfg(wasm)]
    pub(crate) async fn settle(&self) -> Result<(), LateError> {
        let before = self.scopes.lock().next;
        for _ in 0..SETTLE_ROUNDS {
            if self.has_landed(before) {
                break;
            }
            yield_to_event_loop().await;
        }
        self.take()
    }

    /// Whether every scope with a ticket below `before` has landed.
    fn has_landed(&self, before: u64) -> bool {
        let scopes = self.scopes.lock();
        scopes.open.first().is_none_or(|ticket| *ticket >= before)
    }

    /// Reports what `caught` resolves to, when it resolves.
    ///
    /// Every error that lands goes to the log as an error, which the browser
    /// shows as a `console.error`, so a frame no later call follows still reports it.
    /// It also runs `on_error`, and the cause waits in the slot for the next call unless
    /// `failure` says the call already failed and returned its own error. The errors the cause led to only log.
    #[cfg(wasm)]
    pub(crate) fn watch(
        self: &Arc<Self>,
        label: &str,
        caught: impl Future<Output = Option<Caught>> + 'static,
        failure: Option<String>,
        on_error: impl FnOnce() + 'static,
    ) {
        let slot = self.clone();
        let label = label.to_string();
        let ticket = self.open();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(Caught { cause, symptoms }) = caught.await {
                on_error();
                let late = |error: wgpu::Error| LateError {
                    label: label.clone(),
                    message: error.to_string(),
                };
                let error = late(cause);
                match failure {
                    Some(failure) => log::error!(
                        "{}. The call had already failed: {failure}",
                        RendererError::from(error)
                    ),
                    None => {
                        log::error!("{}", RendererError::from(error.clone()));
                        slot.store(error);
                    }
                }
                for symptom in symptoms {
                    let symptom = RendererError::from(late(symptom));
                    log::error!("{symptom}. It follows from the error before it");
                }
            }
            slot.close(ticket);
        });
    }

    /// Records a scope whose error hasn't landed yet and returns its ticket.
    fn open(&self) -> u64 {
        let mut scopes = self.scopes.lock();
        let ticket = scopes.next;
        scopes.next += 1;
        scopes.open.insert(ticket);
        ticket
    }

    /// Marks the scope with `ticket` as landed.
    fn close(&self, ticket: u64) {
        self.scopes.lock().open.remove(&ticket);
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

    // Story: a readback waits for the scopes opened before it, in any order they land,
    // and a scope opened after it began doesn't hold it up.
    #[test]
    fn waits_for_earlier_scopes_only() {
        // Arrange
        let slot = ErrorSlot::default();
        let first = slot.open();
        let second = slot.open();
        let before = slot.scopes.lock().next;
        let later = slot.open();

        // Act
        let waiting = slot.has_landed(before);
        slot.close(second);
        let still_waiting = slot.has_landed(before);
        slot.close(first);
        let done = slot.has_landed(before);

        // Assert
        assert!(!waiting && !still_waiting, "earlier scopes are still open");
        assert!(done, "only the later scope {later} is open");
    }

    // Story: render and a readback report a late error as the same typed
    // ValidationError, with the same label and the same message.
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
        assert!(
            matches!(&texture, TextureError::ValidationError { label, .. } if label == "frame"),
            "got {texture:?}"
        );
        assert_eq!(texture.to_string(), renderer.to_string());
    }
}
