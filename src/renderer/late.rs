//! GPU errors that a wasm frame reports after `render` has returned.

#[cfg(wasm)]
use super::scope::{Caught, Fault};
use super::{ErrorKind, RendererError};
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
    pub(crate) kind: ErrorKind,
    pub(crate) label: String,
    pub(crate) message: String,
}

impl From<LateError> for RendererError {
    fn from(error: LateError) -> Self {
        RendererError::DeviceError {
            kind: error.kind,
            label: error.label,
            message: error.message,
        }
    }
}

impl From<LateError> for TextureError {
    fn from(error: LateError) -> Self {
        TextureError::DeviceError {
            kind: error.kind,
            label: error.label,
            message: error.message,
        }
    }
}

/// Holds the first late error until the next call that reports it takes it.
///
/// The error keeps the ticket of the scope that stored it. A call that waits for
/// the scopes opened before it takes only an error one of those scopes stored,
/// so an error from a frame rendered while it waits stays for the next call.
#[derive(Debug, Default)]
pub(crate) struct ErrorSlot {
    first: Mutex<Option<Stored>>,
    scopes: Mutex<OpenScopes>,
}

/// A late error and the ticket of the scope that caught it.
#[derive(Debug)]
struct Stored {
    ticket: u64,
    error: LateError,
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
    /// Keeps `error`, caught by the scope with `ticket`, unless an earlier one is still waiting.
    fn store(&self, ticket: u64, error: LateError) {
        self.first.lock().get_or_insert(Stored { ticket, error });
    }

    /// Returns the waiting error, once, and empties the slot.
    pub(crate) fn take(&self) -> Result<(), LateError> {
        match self.first.lock().take() {
            Some(stored) => Err(stored.error),
            None => Ok(()),
        }
    }

    /// Returns the waiting error, once, when a scope with a ticket below `before` stored it.
    /// An error from a later scope stays in the slot.
    fn take_before(&self, before: u64) -> Result<(), LateError> {
        let mut first = self.first.lock();
        match first.take_if(|stored| stored.ticket < before) {
            Some(stored) => Err(stored.error),
            None => Ok(()),
        }
    }

    /// Waits until every scope opened before this call has landed, then returns
    /// the error one of them stored, once. An error from a later scope stays for the next call.
    #[cfg(wasm)]
    pub(crate) async fn settle(&self) -> Result<(), LateError> {
        let before = self.scopes.lock().next;
        for _ in 0..SETTLE_ROUNDS {
            if self.has_landed(before) {
                break;
            }
            yield_to_event_loop().await;
        }
        self.take_before(before)
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
                let late = |fault: Fault| LateError {
                    kind: fault.kind,
                    label: label.clone(),
                    message: fault.error.to_string(),
                };
                let error = late(cause);
                match failure {
                    Some(failure) => log::error!(
                        "{}. The call had already failed: {failure}",
                        RendererError::from(error)
                    ),
                    None => {
                        log::error!("{}", RendererError::from(error.clone()));
                        slot.store(ticket, error);
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
            kind: ErrorKind::Validation,
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
        slot.store(0, late("render pipeline"));
        slot.store(1, late("frame"));

        // Assert
        assert_eq!(slot.take(), Err(late("render pipeline")));
    }

    // Story: an error is reported once, and the slot takes a new one afterwards.
    #[test]
    fn reports_each_error_once() {
        // Arrange
        let slot = ErrorSlot::default();
        slot.store(0, late("frame"));

        // Act
        let first = slot.take();
        let second = slot.take();
        slot.store(1, late("compute pipeline"));
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

    // Story: a readback takes only an error that a scope opened before it stored.
    // A frame rendered while it waits keeps its error for the next call.
    #[test]
    fn settles_only_earlier_errors() {
        // Arrange
        let slot = ErrorSlot::default();
        let earlier = slot.open();
        let before = slot.scopes.lock().next;
        let later = slot.open();
        slot.store(later, late("frame"));

        // Act
        let waiting = slot.take_before(before);
        let next = slot.take();
        slot.store(earlier, late("render pipeline"));
        let own = slot.take_before(before);

        // Assert
        assert_eq!(waiting, Ok(()), "the later frame's error waits");
        assert_eq!(next, Err(late("frame")), "the next call takes it");
        assert_eq!(own, Err(late("render pipeline")));
    }

    // Story: render and a readback report a late error as the same typed
    // DeviceError, with the same kind, label and message.
    #[test]
    fn converts_to_each_call_error() {
        // Arrange
        let error = LateError {
            kind: ErrorKind::Memory,
            ..late("frame")
        };

        // Act
        let renderer = RendererError::from(error.clone());
        let texture = TextureError::from(error);

        // Assert
        assert!(
            matches!(&renderer, RendererError::DeviceError { kind: ErrorKind::Memory, label, .. } if label == "frame"),
            "got {renderer:?}"
        );
        assert!(
            matches!(&texture, TextureError::DeviceError { kind: ErrorKind::Memory, label, .. } if label == "frame"),
            "got {texture:?}"
        );
        assert_eq!(texture.to_string(), renderer.to_string());
    }
}
