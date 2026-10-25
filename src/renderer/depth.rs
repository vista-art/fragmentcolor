//! The depth attachment of a render pass, and how its pipelines use it.

use super::{Lease, RenderContext, RendererError, TextureKey};
use crate::material::AlphaMode;
use crate::{PassObject, TextureObject};

/// How a render pipeline uses its pass's depth attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum DepthUse {
    /// The pass has no depth attachment.
    Off,
    /// Nearer fragments win, and opaque ones write depth.
    Test(wgpu::TextureFormat),
    /// Every fragment passes, and none writes depth.
    Ignore(wgpu::TextureFormat),
}

impl DepthUse {
    /// The pipeline's depth state for `alpha_mode`.
    /// Blended fragments test depth but never write it, so they hide nothing drawn later.
    pub(super) fn state(self, alpha_mode: AlphaMode) -> Option<wgpu::DepthStencilState> {
        let (format, compare, write) = match self {
            Self::Off => return None,
            Self::Test(format) => (
                format,
                wgpu::CompareFunction::LessEqual,
                matches!(alpha_mode, AlphaMode::Opaque | AlphaMode::Mask),
            ),
            Self::Ignore(format) => (format, wgpu::CompareFunction::Always, false),
        };
        Some(wgpu::DepthStencilState {
            format,
            depth_write_enabled: Some(write),
            depth_compare: Some(compare),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    }
}

/// The depth attachment of one render pass.
pub(super) enum PassDepth<'pool> {
    /// The pass draws in order, without depth.
    Off,
    /// The texture the app set with `Pass::set_depth_target`.
    Target {
        view: wgpu::TextureView,
        format: wgpu::TextureFormat,
    },
    /// A pooled texture for a pass that draws models.
    Scene(Lease<'pool>),
}

impl PassDepth<'_> {
    /// The attachment the render pass begins with. Depth clears on every pass.
    pub(super) fn attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> {
        let (view, store) = match self {
            Self::Off => return None,
            Self::Target { view, .. } => (view, wgpu::StoreOp::Store),
            Self::Scene(lease) => (lease.view(), wgpu::StoreOp::Discard),
        };
        Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store,
            }),
            stencil_ops: None,
        })
    }

    /// How a shader's pipeline uses this attachment.
    /// A fullscreen shader ignores a scene depth, so a backdrop drawn first hides no model.
    pub(super) fn pick(&self, fullscreen: bool) -> DepthUse {
        match self {
            Self::Off => DepthUse::Off,
            Self::Target { format, .. } => DepthUse::Test(*format),
            Self::Scene(_) if fullscreen => DepthUse::Ignore(TextureObject::DEPTH_FORMAT),
            Self::Scene(_) => DepthUse::Test(TextureObject::DEPTH_FORMAT),
        }
    }
}

impl RenderContext {
    /// The depth attachment `pass` draws with, sized and sampled like its color attachment.
    ///
    /// A depth target the app set wins. Otherwise a pass that draws models
    /// takes a pooled depth texture, and any other pass draws without depth.
    pub(super) fn resolve_depth(
        &self,
        pass: &PassObject,
        extent: wgpu::Extent3d,
        sample_count: u32,
    ) -> Result<PassDepth<'_>, RendererError> {
        let target = pass.depth_target.read().as_ref().map(|target| *target.id());
        if let Some(id) = target {
            let Some(texture) = self.get_texture(&id) else {
                return Ok(PassDepth::Off);
            };
            let depth = texture.inner.sample_count();
            if depth != sample_count {
                return Err(RendererError::DepthSampleCountMismatch {
                    depth,
                    pass: sample_count,
                });
            }
            let view = texture.create_view();
            let format = texture.format();
            return Ok(PassDepth::Target { view, format });
        }
        let models = !pass.model_entries.read().is_empty();
        if !models || extent.width == 0 || extent.height == 0 {
            return Ok(PassDepth::Off);
        }
        let size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..extent
        };
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let key = TextureKey::new(size, TextureObject::DEPTH_FORMAT, sample_count, usage);
        Ok(PassDepth::Scene(Lease::take(
            &self.texture_pool,
            &self.device,
            key,
        )))
    }
}
