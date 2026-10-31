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

use crate::MobileWindowTarget;
use crate::core::renderer::platform::metal_layer::{check_layer, read_side};
use crate::{Renderer, WindowTarget};

use crate::error::platform::mobile::FragmentColorError;

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
    //
    // Foreign bindings see this as `createTarget(metalLayerPtr:)`. A
    // `Renderer+Extensions.swift` helper accepts a `CAMetalLayer` directly
    // and extracts the pointer.
    #[uniffi::method(name = "createTarget")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_target_ios.md")]
    pub fn create_target_ios(
        self: Arc<Self>,
        metal_layer_ptr: u64,
    ) -> Result<Arc<MobileWindowTarget>, FragmentColorError> {
        let (target, extent) = read_layer(metal_layer_ptr)?;
        let (context, surface, config) =
            pollster::block_on(self.configure_unsafe_surface(target, extent))?;
        Ok(MobileWindowTarget::new(WindowTarget::new(
            context, surface, config,
        )))
    }
}

/// The surface target and drawable size of the `CAMetalLayer` at `pointer`.
fn read_layer(
    pointer: u64,
) -> Result<(wgpu::SurfaceTargetUnsafe, wgpu::Extent3d), FragmentColorError> {
    let layer = pointer as *mut AnyObject;
    // SAFETY: Swift passes the address of a live Objective-C object, and `as_ref`
    // turns a null pointer into `None`. No other address can be checked here.
    let Some(object) = (unsafe { layer.as_ref() }) else {
        return Err(FragmentColorError::Init(
            "createTarget: the CAMetalLayer pointer is null".into(),
        ));
    };
    // Only a CAMetalLayer answers drawableSize, so check the class before asking.
    check_layer(object)?;
    let size: CGSize = unsafe { msg_send![layer, drawableSize] };
    let extent = wgpu::Extent3d {
        width: read_side(size.width)?,
        height: read_side(size.height)?,
        depth_or_array_layers: 1,
    };
    let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer.cast());
    Ok((target, extent))
}
