//! The error scopes GPU work runs in, and which of their errors a call reports.

use super::ErrorKind;
#[cfg(not(wasm))]
use super::RendererError;
#[cfg(not(wasm))]
use std::cell::Cell;

/// The kinds of GPU error each error scope catches, outermost first,
/// each with the kind its errors report.
pub(super) const SCOPES: [(wgpu::ErrorFilter, ErrorKind); 3] = [
    (wgpu::ErrorFilter::OutOfMemory, ErrorKind::Memory),
    (wgpu::ErrorFilter::Internal, ErrorKind::Internal),
    (wgpu::ErrorFilter::Validation, ErrorKind::Validation),
];

/// One error a scope caught, and the kind its filter catches.
pub(crate) struct Fault {
    pub(crate) kind: ErrorKind,
    pub(crate) error: wgpu::Error,
}

/// What one set of error scopes caught: the error that caused the others, and those others.
pub(crate) struct Caught {
    pub(crate) cause: Fault,
    pub(crate) symptoms: Vec<Fault>,
}

impl Caught {
    /// Picks the cause among what each scope caught, given in `SCOPES` order.
    ///
    /// Running out of memory or an internal failure leaves an object invalid,
    /// and each later use of that object raises a validation error, never the reverse.
    /// So the first kind in `SCOPES` order that caught an error is the cause of the rest.
    pub(super) fn split(caught: [Option<wgpu::Error>; 3]) -> Option<Self> {
        let mut faults = SCOPES
            .into_iter()
            .zip(caught)
            .filter_map(|((_, kind), error)| {
                Some(Fault {
                    kind,
                    error: error?,
                })
            });
        let cause = faults.next()?;
        Some(Self {
            cause,
            symptoms: faults.collect(),
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
        log::warn!(
            "{label}: the GPU then reported ({}): {}",
            symptom.kind,
            symptom.error
        );
    }
    let cause_text = cause.error.to_string();
    let scoped = RendererError::DeviceError {
        kind: cause.kind,
        label: label.to_string(),
        detail: cause_text.clone(),
    };
    match (result, origin) {
        (Err(error), Origin::Cpu) => {
            log::warn!("{label} failed: {error}. The GPU also reported: {cause_text}");
            (Err(error), Origin::Cpu)
        }
        (Err(error), Origin::Gpu) => {
            log::warn!("{label} failed: {cause_text}. It then caused: {error}");
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
        assert!(matches!(first.cause.error, wgpu::Error::OutOfMemory { .. }));
        assert_eq!(first.symptoms.len(), 2, "both symptoms are kept");
        assert!(matches!(second.cause.error, wgpu::Error::Internal { .. }));
        assert!(matches!(
            second.symptoms[..],
            [Fault {
                error: wgpu::Error::Validation { .. },
                ..
            }]
        ));
        assert!(Caught::split([None, None, None]).is_none());
    }

    // Story: each error reports the kind of the scope that caught it,
    // so a frame that ran out of memory never reads as a validation error.
    #[test]
    fn each_scope_reports_its_kind() {
        // Arrange
        let every = [Some(memory()), Some(internal()), Some(validation())];
        let frame = Caught::split([Some(memory()), None, Some(validation())]);

        // Act
        let caught = Caught::split(every).expect("three errors");
        let (result, _) = resolve::<()>("frame", Ok(()), Origin::Cpu, frame);

        // Assert
        let kinds: Vec<ErrorKind> = std::iter::once(&caught.cause)
            .chain(&caught.symptoms)
            .map(|fault| fault.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                ErrorKind::Memory,
                ErrorKind::Internal,
                ErrorKind::Validation
            ]
        );
        assert!(
            matches!(&result, Err(RendererError::DeviceError { kind: ErrorKind::Memory, label, .. }) if label == "frame"),
            "got {result:?}"
        );
    }

    // Story: a CPU check that fails with a DeviceError of its own, such as a limit
    // check, still names the input to fix, even when the scope caught a GPU error first.
    #[test]
    fn cpu_error_wins_whatever_its_variant() {
        // Arrange
        let limit = RendererError::DeviceError {
            kind: ErrorKind::Validation,
            label: "storage".into(),
            detail: "past max_buffer_size".into(),
        };
        let caught = Caught::split([None, None, Some(validation())]);

        // Act
        let (result, origin) = resolve::<()>("frame", Err(limit), Origin::Cpu, caught);

        // Assert
        assert!(
            matches!(&result, Err(RendererError::DeviceError { label, .. }) if label == "storage"),
            "got {result:?}"
        );
        assert_eq!(origin, Origin::Cpu);
    }
}
