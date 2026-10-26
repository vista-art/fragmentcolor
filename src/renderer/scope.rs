//! The error scopes GPU work runs in, and which of their errors a call reports.

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
}
