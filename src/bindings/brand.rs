//! The `__fc_kind` brand a JavaScript class carries, so a check knows a FragmentColor
//! object after a minifier renames its class, and the bridge macros that check it.

/// Marker trait that supplies a stable, compile-time kind string.
pub(crate) trait FcKind {
    const KIND: &'static str;
}

/// Whether two kind strings match, in a const context.
pub(crate) const fn same_kind(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Implement FcKind for a type and expose a platform getter/property.
///
/// Usage:
///   impl_fc_kind!(Renderer, "Renderer");
///   impl_fc_kind!(CanvasTarget, "CanvasTarget");
macro_rules! impl_fc_kind {
    ($ty:ty, $name:literal) => {
        impl $crate::FcKind for $ty {
            const KIND: &'static str = $name;
        }

        // JavaScript: expose a non-configurable prototype getter named "__fc_kind".
        #[cfg(wasm)]
        #[wasm_bindgen]
        impl $ty {
            #[wasm_bindgen(getter, js_name = "__fc_kind")]
            pub fn js_fc_kind(&self) -> String {
                <Self as $crate::FcKind>::KIND.into()
            }
        }
    };
}
pub(crate) use impl_fc_kind;

// One-liner for WASM: implement TryFrom<&JsValue> for a wasm_bindgen class through
// wasm-bindgen's public `TryFromJsValue`. That trait takes the JS object it reads, so the
// class hands out a fresh handle to a clone, `__fc_share`, and the bridge takes that one.
// The caller's object stays usable.
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

                // The brand survives minifiers, which rename classes.
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
pub(crate) use impl_tryfrom_js_ref_anchor;

// Bridge macro: implement both TryFrom<&JsValue> (anchor) and owned-via-ref in one line.
macro_rules! impl_js_bridge {
    ($t:ident, $err:ty) => {
        $crate::impl_js_bridge!($t, $err, $t);
    };
    ($t:ty, $err:ty, $class:ident) => {
        // A bridged type must carry the brand its check reads, in every build.
        const _: () = assert!(
            $crate::bindings::brand::same_kind(<$t as $crate::FcKind>::KIND, stringify!($class)),
            "a type that goes through impl_js_bridge! needs impl_fc_kind! with the same name",
        );
        $crate::impl_tryfrom_js_ref_anchor!($t, $err, $class);
        #[cfg(wasm)]
        $crate::impl_tryfrom_owned_via_ref!($t, wasm_bindgen::JsValue, $err);
    };
}
pub(crate) use impl_js_bridge;
