use crate::{Target, TargetFrame, WindowTarget};
use core_graphics::geometry::CGSize;
use objc::*;

/// iOS-specific target that wraps a WindowTarget.
#[doc(hidden)]
#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct IosTarget(WindowTarget);

#[doc(hidden)]
#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct IosTextureTarget(crate::TextureTarget);

impl Target for IosTarget {
    fn size(&self) -> crate::Size {
        self.0.size()
    }
    fn resize(&mut self, size: impl Into<crate::Size>) {
        self.0.resize(size.into());
    }
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
}

impl Target for IosTextureTarget {
    fn size(&self) -> crate::Size {
        self.0.size()
    }
    fn resize(&mut self, size: impl Into<crate::Size>) {
        self.0.resize(size.into());
    }
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
    fn get_image(&self) -> Vec<u8> {
        <crate::TextureTarget as Target>::get_image(&self.0)
    }
}

#[cfg_attr(mobile, uniffi::export)]
impl crate::Renderer {
    /// Creates a new Renderer (iOS wrapper variant)
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new_ios() -> Self {
        Self::new()
    }

    /// Create a target from a CAMetalLayer pointer (as u64) on iOS.
    /// The pointer must remain valid for the lifetime of the target.
    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target_ios(
        &self,
        metal_layer_ptr: u64,
    ) -> Result<IosTarget, crate::InitializationError> {
        // Read drawable size from CAMetalLayer
        let metal_layer = metal_layer_ptr as *mut objc::runtime::Object;
        let (width, height) = unsafe {
            let size: CGSize = objc::msg_send![metal_layer, drawableSize];
            (size.width as u32, size.height as u32)
        };
        let size = wgpu::Extent3d {
            width: u32::max(width, 1),
            height: u32::max(height, 1),
            depth_or_array_layers: 1,
        };

        // Build unsafe surface from the CAMetalLayer
        let instance = crate::renderer::platform::all::create_instance().await;
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                metal_layer as _,
            ))?
        };

        // Use core helper to configure and produce a WindowTarget
        let (context, surface, config) = self
            .create_surface(wgpu::SurfaceTarget::Surface(surface), size)
            .await?;

        Ok(IosTarget(WindowTarget::new(context, surface, config)))
    }

    /// Headless texture target (iOS wrapper variant)
    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target_ios(
        &self,
        size: impl Into<crate::Size>,
    ) -> Result<IosTextureTarget, crate::InitializationError> {
        let target = self.create_texture_target(size).await?;
        Ok(IosTextureTarget(target))
    }

    /// Render wrapper (iOS variant)
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_ios(
        &self,
        renderable: &impl crate::renderer::Renderable,
        target: &impl crate::Target,
    ) -> Result<(), crate::RendererError> {
        self.render(renderable, target)
    }
}
