//! The error scopes GPU work runs in, and which of their errors a call reports.

#[cfg(not(wasm))]
use super::RendererError;
#[cfg(not(wasm))]
use std::cell::Cell;

/// The kinds of GPU error each error scope catches, outermost first.
pub(super) const SCOPES: [wgpu::ErrorFilter; 3] = [
    wgpu::ErrorFilter::OutOfMemory,
    wgpu::ErrorFilter::Internal,
    wgpu::ErrorFilter::Validation,
];

/// What one set of error scopes caught: the error that caused the others, and those others.
pub(crate) struct Caught {
    pub(crate) cause: wgpu::Error,
    pub(crate) symptoms: Vec<wgpu::Error>,
}

impl Caught {
    /// Picks the cause among what each scope caught, given in `SCOPES` order.
    ///
    /// Running out of memory or an internal failure leaves an object invalid,
    /// and each later use of that object raises a validation error, never the reverse.
    /// So the first kind in `SCOPES` order that caught an error is the cause of the rest.
    pub(super) fn split(caught: [Option<wgpu::Error>; 3]) -> Option<Self> {
        let mut errors = caught.into_iter().flatten();
        let cause = errors.next()?;
        Some(Self {
            cause,
            symptoms: errors.collect(),
        })
    }
}

/// Where the error that a scope's work returned came from.
///
/// A nested scope's catch and a surface that failed validation are `Gpu`: the GPU
/// raised each one, and an outer scope may hold the error that caused it. Anything else,
/// such as a check of the input, is `Cpu`. wgpu keeps error scopes per thread, and so does this.
#[cfg(not(wasm))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Cpu,
    Gpu,
}

#[cfg(not(wasm))]
thread_local! {
    static ORIGIN: Cell<Origin> = const { Cell::new(Origin::Cpu) };
}

#[cfg(not(wasm))]
impl Origin {
    /// Records this as where the error the running work returns came from.
    pub(super) fn record(self) {
        ORIGIN.set(self);
    }

    /// Returns the origin recorded last, and resets it to `Cpu`.
    pub(super) fn take() -> Self {
        ORIGIN.replace(Origin::Cpu)
    }
}

/// Picks what a scope returns on native, and where that came from.
///
/// `result` is what the scope's work returned, its error from `origin`, and `caught` is what
/// the scope's own filters caught. The work stops at its first error, so a caught error came first.
/// A GPU error from the work then follows from it, and the caught cause returns under `label`.
/// A CPU error names the input to fix, so it returns. Every error not returned goes to the log.
#[cfg(not(wasm))]
pub(super) fn resolve<T>(
    label: &str,
    result: Result<T, RendererError>,
    origin: Origin,
    caught: Option<Caught>,
) -> (Result<T, RendererError>, Origin) {
    let Some(Caught { cause, symptoms }) = caught else {
        let origin = if result.is_err() { origin } else { Origin::Cpu };
        return (result, origin);
    };
    for symptom in symptoms {
        log::warn!("{label}: the GPU then reported: {symptom}");
    }
    let scoped = RendererError::ValidationError {
        label: label.to_string(),
        message: cause.to_string(),
    };
    match (result, origin) {
        (Err(error), Origin::Cpu) => {
            log::warn!("{label} failed: {error}. The GPU also reported: {cause}");
            (Err(error), Origin::Cpu)
        }
        (Err(error), Origin::Gpu) => {
            log::warn!("{label} failed: {cause}. It then caused: {error}");
            (Err(scoped), Origin::Gpu)
        }
        (Ok(_), _) => (Err(scoped), Origin::Gpu),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validation() -> wgpu::Error {
        wgpu::Error::Validation {
            source: Box::new(std::io::Error::other("invalid buffer")),
            description: "the buffer is invalid".into(),
        }
    }

    fn internal() -> wgpu::Error {
        wgpu::Error::Internal {
            source: Box::new(std::io::Error::other("generation")),
            description: "the backend compiler failed".into(),
        }
    }

    fn memory() -> wgpu::Error {
        wgpu::Error::OutOfMemory {
            source: Box::new(std::io::Error::other("out of memory")),
        }
    }

    // Story: a buffer that ran out of memory is invalid, and writing to it raises a
    // validation error. The out of memory error is the cause, then an internal error.
    #[test]
    fn cause_comes_before_its_symptoms() {
        // Arrange
        let every = [Some(memory()), Some(internal()), Some(validation())];
        let later = [None, Some(internal()), Some(validation())];

        // Act
        let first = Caught::split(every).expect("three errors");
        let second = Caught::split(later).expect("two errors");

        // Assert
        assert!(matches!(first.cause, wgpu::Error::OutOfMemory { .. }));
        assert_eq!(first.symptoms.len(), 2, "both symptoms are kept");
        assert!(matches!(second.cause, wgpu::Error::Internal { .. }));
        assert!(matches!(
            second.symptoms[..],
            [wgpu::Error::Validation { .. }]
        ));
        assert!(Caught::split([None, None, None]).is_none());
    }

    // Story: a CPU check that fails with a ValidationError of its own, such as a limit
    // check, still names the input to fix, even when the scope caught a GPU error first.
    #[test]
    fn cpu_error_wins_whatever_its_variant() {
        // Arrange
        let limit = RendererError::ValidationError {
            label: "storage".into(),
            message: "past max_buffer_size".into(),
        };
        let caught = Caught::split([None, None, Some(validation())]);

        // Act
        let (result, origin) = resolve::<()>("frame", Err(limit), Origin::Cpu, caught);

        // Assert
        assert!(
            matches!(&result, Err(RendererError::ValidationError { label, .. }) if label == "storage"),
            "got {result:?}"
        );
        assert_eq!(origin, Origin::Cpu);
    }
}
