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

// One-liner for WASM: implement TryFrom<&JsValue> for a wasm_bindgen class through
// wasm-bindgen's public `TryFromJsValue`. That trait takes the JS object it reads, so the
// class hands out a fresh handle to a clone, `__fc_share`, and the bridge takes that one.
// The caller's object stays usable, and no wasm-bindgen ABI detail is named here.
#[macro_export]
macro_rules! impl_tryfrom_js_ref_anchor {
    ($t:ty, $err:ty, $class:ident) => {
        #[cfg(wasm)]
        #[wasm_bindgen::prelude::wasm_bindgen(js_class = $class)]
        impl $t {
            #[doc(hidden)]
            #[wasm_bindgen(js_name = "__fc_share")]
            pub fn js_fc_share(&self) -> $t {
                self.clone()
            }
        }

        #[cfg(wasm)]
        impl TryFrom<&wasm_bindgen::JsValue> for $t {
            type Error = $err;
            fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
                use js_sys::{Function, Reflect};
                use wasm_bindgen::JsCast;
                use wasm_bindgen::convert::TryFromJsValue;

                // The brand survives minifiers, which rename classes. A check by
                // constructor name passed in development builds and failed in production.
                let brand = Reflect::get(value, &wasm_bindgen::JsValue::from_str("__fc_kind"))
                    .ok()
                    .and_then(|brand| brand.as_string());
                let name = stringify!($class);
                if brand.as_deref() != Some(name) {
                    return Err(<$err>::Error(format!("Type mismatch: expected {name}")));
                }
                let share = Reflect::get(value, &wasm_bindgen::JsValue::from_str("__fc_share"))
                    .ok()
                    .and_then(|share| share.dyn_into::<Function>().ok());
                let fresh = share.and_then(|share| share.call0(value).ok());
                match fresh.and_then(|fresh| <$t as TryFromJsValue>::try_from_js_value(fresh).ok())
                {
                    Some(shared) => Ok(shared),
                    None => Err(<$err>::Error(format!("{name} has no handle to share"))),
                }
            }
        }
    };
}

// Bridge macro: implement both TryFrom<&JsValue> (anchor) and owned-via-ref in one line.
#[macro_export]
macro_rules! impl_js_bridge {
    ($t:ident, $err:ty) => {
        $crate::impl_js_bridge!($t, $err, $t);
    };
    ($t:ty, $err:ty, $class:ident) => {
        // A bridged type must carry the brand its check reads, in every build.
        const _: () = assert!(
            $crate::fc_kind::same_kind(<$t as $crate::FcKind>::KIND, stringify!($class)),
            "a type that goes through impl_js_bridge! needs impl_fc_kind! with the same name",
        );
        $crate::impl_tryfrom_js_ref_anchor!($t, $err, $class);
        #[cfg(wasm)]
        $crate::impl_tryfrom_owned_via_ref!($t, wasm_bindgen::JsValue, $err);
    };
}
