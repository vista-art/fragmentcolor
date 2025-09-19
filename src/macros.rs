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

// -------------------------------------------------------------------------------------------------
// Extended (bidirectional) variants
// -------------------------------------------------------------------------------------------------
// These new macros produce FOUR implementations (owned + &ref in BOTH directions) in one shot.
// They require the caller to provide the conversion logic as closures / function paths so we do not
// attempt to guess how to convert between the two distinct types. This avoids accidental infinite
// recursion that would arise if we tried to delegate each direction to the other automatically.
//
// Design notes:
// * Both types must implement Clone so we can derive the &T conversions from the owned ones.
// * For From: you provide two closures (A -> B) and (B -> A).
// * For TryFrom: you provide two closures (A -> Result<B, E>) and (B -> Result<A, E>). The same
//   error type E is used for all four implementations.
// * Each closure/expression is evaluated exactly once inside its corresponding impl.
//
// Example usage:
//
// impl_from_bidirectional_with_refs!(TypeA, TypeB, |a: TypeA| TypeB::new(a.x), |b: TypeB| TypeA::from(b.y));
// impl_tryfrom_bidirectional_with_refs!(TypeA, TypeB, ConvertError,
//     |a: TypeA| TypeB::try_new(a.x),
//     |b: TypeB| TypeA::try_from_part(b.y)
// );
//
// If you already have some of these impls, do NOT invoke these macros (Rust forbids duplicate impls).
// You can keep using the original lightweight macros above when you only need one delegation.

#[macro_export]
macro_rules! impl_from_into_with_refs {
    ($a:ty, $b:ty, $a_to_b:expr, $b_to_a:expr) => {
        impl From<$a> for $b {
            fn from(v: $a) -> $b {
                ($a_to_b)(v)
            }
        }
        impl From<&$a> for $b {
            fn from(v: &$a) -> $b {
                ($a_to_b)(v.clone())
            }
        }
        impl From<$b> for $a {
            fn from(v: $b) -> $a {
                ($b_to_a)(v)
            }
        }
        impl From<&$b> for $a {
            fn from(v: &$b) -> $a {
                ($b_to_a)(v.clone())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tryfrom_tryinto_with_refs {
    ($a:ty, $b:ty, $err:ty, $a_to_b:expr, $b_to_a:expr) => {
        impl TryFrom<$a> for $b {
            type Error = $err;
            fn try_from(v: $a) -> Result<$b, Self::Error> {
                ($a_to_b)(v)
            }
        }
        impl TryFrom<&$a> for $b {
            type Error = $err;
            fn try_from(v: &$a) -> Result<$b, Self::Error> {
                ($a_to_b)(v.clone())
            }
        }
        impl TryFrom<$b> for $a {
            type Error = $err;
            fn try_from(v: $b) -> Result<$a, Self::Error> {
                ($b_to_a)(v)
            }
        }
        impl TryFrom<&$b> for $a {
            type Error = $err;
            fn try_from(v: &$b) -> Result<$a, Self::Error> {
                ($b_to_a)(v.clone())
            }
        }
    };
}
