//! iOS-specific entry points for uniffi.
//!
//! `Renderer::create_target_ios` takes the raw pointer of a `CAMetalLayer`
//! (as `u64`) and returns a fully-configured `WindowTarget`. The Swift side
//! obtains the pointer with `Unmanaged.passUnretained(layer).toOpaque()`.

use std::sync::Arc;

use lsp_doc::lsp_doc;
use objc2::encode::{Encode, Encoding, RefEncode};
use objc2::msg_send;
use objc2::runtime::AnyObject;

use crate::{Renderer, WindowTarget};

use super::FragmentColorError;

/// Mirror of Core Graphics `CGSize`. Defined locally to avoid depending on
/// the `core-graphics` crate.
#[repr(C)]
#[derive(Copy, Clone)]
struct CGSize {
    width: f64,
    height: f64,
}

// SAFETY: `CGSize` is a C struct of two `f64`s, matching the Objective-C
// layout expected by `-[CAMetalLayer drawableSize]`.
unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct(
        "CGSize",
        &[<f64 as Encode>::ENCODING, <f64 as Encode>::ENCODING],
    );
}

unsafe impl RefEncode for CGSize {
    const ENCODING_REF: Encoding = Encoding::Pointer(&<Self as Encode>::ENCODING);
}

#[uniffi::export]
impl Renderer {
    // Exposed as sync because `wgpu::SurfaceTargetUnsafe` is not `Send`
    // (it wraps a raw pointer), so the resulting future can't satisfy
    // uniffi's `Send` bound on async exports. Adapter/device creation is
    // driven by pollster internally.
    #[lsp_doc("docs/api/core/renderer/hidden/create_target_ios.md")]
    pub fn create_target_ios(
        self: Arc<Self>,
        metal_layer_ptr: u64,
    ) -> Result<Arc<WindowTarget>, FragmentColorError> {
        // Read drawableSize off the CAMetalLayer via Objective-C runtime.
        let layer = metal_layer_ptr as *mut AnyObject;
        let size: CGSize = unsafe { msg_send![layer, drawableSize] };
        let extent = wgpu::Extent3d {
            width: u32::max(size.width as u32, 1),
            height: u32::max(size.height as u32, 1),
            depth_or_array_layers: 1,
        };

        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer.cast());
        let (context, surface, config) =
            pollster::block_on(self.configure_unsafe_surface(target, extent))
                .map_err(FragmentColorError::from)?;

        Ok(Arc::new(WindowTarget::new(context, surface, config)))
    }
}
