//! Cross-language stable kind branding.
//!
//! Provides a trait and a helper macro to expose a stable "__fc_kind"
//! property in JavaScript (via wasm-bindgen), so type identity checks
//! remain stable under bundler/minifier mangling.

/// Marker trait that supplies a stable, compile-time kind string.
pub trait FcKind {
    const KIND: &'static str;
}

/// Implement FcKind for a type and expose a platform getter/property.
///
/// Usage:
///   impl_fc_kind!(Renderer, "Renderer");
///   impl_fc_kind!(CanvasTarget, "CanvasTarget");
#[macro_export]
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
