//! Macros that derive the borrowed and owned forms of a `From` or `TryFrom` from one conversion.

// Implement TryFrom<OwnedSrc> for Dst by delegating to an existing TryFrom<&OwnedSrc>.
// We require the caller to specify the Error type to avoid lifetime issues with borrowed sources.
#[cfg(wasm)]
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
#[cfg(wasm)]
pub(crate) use impl_tryfrom_owned_via_ref;

// -------------------------------------------------------------------------------------------------
// Extended (bidirectional) variants
// -------------------------------------------------------------------------------------------------
// This macro produces FOUR implementations (owned + &ref in BOTH directions) in one shot.
// It requires the caller to provide the conversion logic as closures / function paths so we do not
// attempt to guess how to convert between the two distinct types. This avoids accidental infinite
// recursion that would arise if we tried to delegate each direction to the other automatically.
//
// Design notes:
// * Both types must implement Clone so we can derive the &T conversions from the owned ones.
// * You provide two closures (A -> B) and (B -> A).
// * Each closure/expression is evaluated exactly once inside its corresponding impl.
//
// Example usage:
//
// impl_from_into_with_refs!(TypeA, TypeB, |a: TypeA| TypeB::new(a.x), |b: TypeB| TypeA::from(b.y));
//
// If you already have some of these impls, do NOT invoke this macro (Rust forbids duplicate impls).
// You can keep using the lightweight macro above when you only need one delegation.

macro_rules! impl_from_into_with_refs {
    ($(#[$meta:meta])* $a:ty, $b:ty, $a_to_b:expr, $b_to_a:expr) => {
        $(#[$meta])*
        impl From<$a> for $b {
            fn from(v: $a) -> $b {
                ($a_to_b)(v)
            }
        }
        $(#[$meta])*
        impl From<&$a> for $b {
            fn from(v: &$a) -> $b {
                ($a_to_b)(v.clone())
            }
        }
        $(#[$meta])*
        impl From<$b> for $a {
            fn from(v: $b) -> $a {
                ($b_to_a)(v)
            }
        }
        $(#[$meta])*
        impl From<&$b> for $a {
            fn from(v: &$b) -> $a {
                ($b_to_a)(v.clone())
            }
        }
    };
}
pub(crate) use impl_from_into_with_refs;
