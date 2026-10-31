//! The `__fc_kind` brand a JavaScript class carries,
//! so a check knows a FragmentColor object after a minifier renames its class.

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
