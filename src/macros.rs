//! Helper macros to implement reference-based conversions without duplicating logic.
//! These macros forward From<&T> and TryFrom<T> to existing implementations.

#[macro_export]
/// Implement From<&Src> for Dst by delegating to the existing From<Src>.
/// Requires Src: Clone (or Copy).
macro_rules! impl_from_ref {
    ($dst:ty, $src:ty) => {
        impl From<&$src> for $dst {
            fn from(v: &$src) -> $dst {
                <$dst as From<$src>>::from(v.clone())
            }
        }
    };
}

// Implement TryFrom<OwnedSrc> for Dst by delegating to an existing TryFrom<&OwnedSrc>.
// We require the caller to specify the Error type to avoid lifetime issues with borrowed sources.
#[macro_export]
macro_rules! impl_tryfrom_owned_via_ref {
    ($dst:ty, $owned_src:ty, $err:ty) => {
        impl TryFrom<$owned_src> for $dst {
            type Error = $err;
            fn try_from(v: $owned_src) -> Result<$dst, Self::Error> {
                <$dst as TryFrom<&$owned_src>>::try_from(&v)
            }
        }
    };
}
