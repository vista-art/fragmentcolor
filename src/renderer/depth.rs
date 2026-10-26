//! The depth attachment of a render pass, and how its pipelines use it.

use super::{Lease, RenderContext, TextureKey};
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

    /// How a shader's pipeline uses this attachment. In a pass that draws models,
    /// `ignores` marks a shader that draws none, such as a backdrop or an overlay:
    /// it draws in order and writes no depth, so it hides no model.
    pub(super) fn pick(&self, ignores: bool) -> DepthUse {
        let format = match self {
            Self::Off => return DepthUse::Off,
            Self::Target { format, .. } => *format,
            Self::Scene(_) => TextureObject::DEPTH_FORMAT,
        };
        match ignores {
            true => DepthUse::Ignore(format),
            false => DepthUse::Test(format),
        }
    }
}

impl RenderContext {
    /// The depth attachment `pass` draws with, sized and sampled like its color attachment.
    ///
    /// The depth `target` the app set wins. `check_attachment` has matched its size and
    /// sample count to the color attachment's. Otherwise a pass that draws models
    /// takes a pooled depth texture, and any other pass draws without depth.
    pub(super) fn resolve_depth(
        &self,
        pass: &PassObject,
        target: Option<&TextureObject>,
        extent: wgpu::Extent3d,
        sample_count: u32,
    ) -> PassDepth<'_> {
        if let Some(texture) = target {
            let view = texture.create_view();
            let format = texture.format();
            return PassDepth::Target { view, format };
        }
        let models = !pass.model_entries.read().is_empty();
        if !models || extent.width == 0 || extent.height == 0 {
            return PassDepth::Off;
        }
        let size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..extent
        };
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let key = TextureKey::new(size, TextureObject::DEPTH_FORMAT, sample_count, usage);
        PassDepth::Scene(Lease::take(&self.texture_pool, &self.device, key))
    }
}
