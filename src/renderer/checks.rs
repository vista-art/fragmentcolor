//! The checks a render runs on the CPU before it records a pass.
//!
//! Each rule here is one wgpu enforces too. Checking it first returns a typed
//! error from the call that broke it, on the web as well, where WebGPU reports
//! its own errors only after `render` has returned.

use super::access::{Access, Accesses, Writer};
use super::{PassDraws, RenderContext, RendererError};
use crate::material::AlphaMode;
use crate::pass::PassError;
use crate::texture::Registration;
use crate::{
    PassObject, Renderable, ShaderObject, Target, TargetFrame, TextureObject, UniformData,
};
use parking_lot::RwLock;
use std::sync::Arc;

/// The color attachment of a render pass, which its depth target and pipelines must fit.
pub(super) struct Attachment {
    texture: wgpu::Texture,
    format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
    sample_count: u32,
}

impl Attachment {
    /// The texture a pass set with `Pass::set_target`.
    pub(super) fn from_target(object: &TextureObject) -> Self {
        Self {
            texture: object.inner.clone(),
            format: object.format,
            size: object.inner.size(),
            sample_count: object.inner.sample_count(),
        }
    }

    /// The frame a pass draws into when it sets no target, at `sample_count`.
    pub(super) fn from_frame(frame: &dyn TargetFrame, sample_count: u32) -> Self {
        Self {
            texture: frame.view().texture().clone(),
            format: frame.format(),
            size: frame.size(),
            sample_count,
        }
    }
}

/// The textures a render pass set to draw into, looked up before the frame.
#[derive(Default)]
pub(super) struct PassTargets {
    /// The color target, or `None` to draw into the frame.
    pub(super) color: Option<Arc<TextureObject>>,
    /// The depth target, or `None` for the pooled depth or none.
    pub(super) depth: Option<Arc<TextureObject>>,
}

/// The scalar type and the channel count a fragment output or a color format holds.
#[derive(Clone, Copy)]
struct Channels {
    scalar: naga::Scalar,
    count: u8,
}

impl std::fmt::Display for Channels {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.scalar.kind {
            naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat => "f",
            naga::ScalarKind::Uint => "u",
            naga::ScalarKind::Sint | naga::ScalarKind::AbstractInt => "i",
            naga::ScalarKind::Bool => "bool",
        };
        let bits = u32::from(self.scalar.width) * 8;
        match self.count {
            1 => write!(formatter, "{prefix}{bits}"),
            count => write!(formatter, "vec{count}<{prefix}{bits}>"),
        }
    }
}

/// The error a render returns on a Renderer that has no context yet. Such a Renderer
/// created no target, so a render pass that draws into one draws into another Renderer's.
pub(super) fn refuse_without_context(
    renderable: &impl Renderable,
    target: &impl Target,
) -> RendererError {
    let draws = renderable.passes().iter().any(|pass| !pass.is_compute());
    match draws && target.owner().is_some() {
        true => RendererError::ForeignTarget,
        false => RendererError::NoContext,
    }
}

impl RenderContext {
    /// Refuses a target another Renderer created, since its frames belong to another device.
    pub(super) fn check_owner(&self, target: &impl Target) -> Result<(), RendererError> {
        match target.owner() {
            Some(owner) if !std::ptr::eq(Arc::as_ptr(&owner), self) => {
                Err(RendererError::ForeignTarget)
            }
            _ => Ok(()),
        }
    }

    /// Checks a compute pass before anything records: its dispatch fits
    /// the device, and no shader writes a texture it also reads. A device
    /// without compute shaders has a limit of 0, and its pipeline build says why.
    pub(super) fn check_compute(&self, pass: &PassObject) -> Result<(), RendererError> {
        let limit = self.device.limits().max_compute_workgroups_per_dimension;
        let (x, y, z) = pass.compute_dispatch();
        if limit > 0 && [x, y, z].into_iter().any(|count| count > limit) {
            return Err(PassError::DispatchLimit {
                pass: pass.name.to_string(),
                dispatch: [x, y, z],
                limit,
            }
            .into());
        }
        // Each shader dispatches in a compute pass of its own.
        for shader in pass.shaders.read().iter() {
            self.add_bindings(&mut Accesses::new(&pass.name), shader)?;
        }
        Ok(())
    }

    /// Looks up the textures `pass` draws into. One this context doesn't hold,
    /// because `unregister_texture` removed it or another Renderer created it,
    /// returns `TextureNotFoundError`.
    pub(super) fn find_targets(&self, pass: &PassObject) -> Result<PassTargets, RendererError> {
        Ok(PassTargets {
            color: self.find_target(&pass.color_target)?,
            depth: self.find_target(&pass.depth_target)?,
        })
    }

    /// The texture behind one target slot of a pass, or `None` when the slot is empty.
    fn find_target(
        &self,
        target: &RwLock<Option<Arc<Registration>>>,
    ) -> Result<Option<Arc<TextureObject>>, RendererError> {
        let id = target
            .read()
            .as_ref()
            .map(|registration| *registration.id());
        id.map(|id| {
            self.get_texture(&id)
                .ok_or(RendererError::TextureNotFoundError(id))
        })
        .transpose()
    }

    /// Checks the plan of `pass` against its color attachment, before the pass records.
    ///
    /// The depth target matches the attachment's size and sample count. Each shader
    /// writes the values the attachment's format takes, and blends only into a format
    /// that blends. No texture the pass draws into is also bound, and no binding writes
    /// a texture another binding of the pass reads.
    pub(super) fn check_attachment(
        &self,
        pass: &PassObject,
        draws: &PassDraws,
        color: &Attachment,
    ) -> Result<(), RendererError> {
        let mut accesses = Accesses::new(&pass.name);
        accesses.attach(&color.texture, Writer::Color)?;
        if let Some(depth) = draws.targets.depth.as_deref() {
            check_depth(depth, color)?;
            accesses.attach(&depth.inner, Writer::Depth)?;
        }
        for plan in draws.shaders.iter() {
            check_output(pass, &plan.shader, color.format)?;
            if *plan.shader.alpha_mode.read() == AlphaMode::Blend {
                self.check_blend(pass, color.format)?;
            }
            self.add_bindings(&mut accesses, &plan.shader)?;
        }
        Ok(())
    }

    /// Refuses `AlphaMode::Blend` into a `format` this device can't blend.
    fn check_blend(
        &self,
        pass: &PassObject,
        format: wgpu::TextureFormat,
    ) -> Result<(), RendererError> {
        let flags = crate::texture::format_features(self, format).flags;
        if flags.contains(wgpu::TextureFormatFeatureFlags::BLENDABLE) {
            return Ok(());
        }
        Err(PassError::InvalidColorTarget(format!(
            "pass '{}' draws into {format:?}, which this device can't blend, and a shader uses AlphaMode::Blend",
            pass.name
        ))
        .into())
    }

    /// Records each registered texture `shader` binds, in binding order.
    fn add_bindings(
        &self,
        accesses: &mut Accesses,
        shader: &ShaderObject,
    ) -> Result<(), RendererError> {
        let mut bindings: Vec<_> = {
            let storage = shader.storage.read();
            storage
                .uniforms
                .values()
                .filter_map(|(_, _, uniform)| match &uniform.data {
                    UniformData::Texture(meta) if !uniform.name.contains('.') => Some((
                        (uniform.group, uniform.binding),
                        uniform.name.clone(),
                        meta.clone(),
                    )),
                    _ => None,
                })
                .collect()
        };
        bindings.sort_by_key(|(slot, _, _)| *slot);
        for (_, name, meta) in bindings {
            let Some(access) = Access::from_class(&meta.class) else {
                continue;
            };
            if let Some(texture) = self.get_texture(&meta.id) {
                accesses.bind(&texture.inner, access, &name)?;
            }
        }
        Ok(())
    }
}

/// Refuses a depth target whose sample count or size differs from the color attachment's.
fn check_depth(depth: &TextureObject, color: &Attachment) -> Result<(), RendererError> {
    let samples = depth.inner.sample_count();
    if samples != color.sample_count {
        return Err(RendererError::DepthSampleCountMismatch {
            depth: samples,
            pass: color.sample_count,
        });
    }
    let size = depth.inner.size();
    let pass = [color.size.width, color.size.height];
    if [size.width, size.height] != pass {
        return Err(RendererError::DepthSizeMismatch {
            depth: [size.width, size.height],
            pass,
        });
    }
    Ok(())
}

/// Refuses a shader whose `@location(0)` output a `format` attachment can't take.
/// It takes the format's scalar kind, at least as wide, in at least as many channels.
fn check_output(
    pass: &PassObject,
    shader: &ShaderObject,
    format: wgpu::TextureFormat,
) -> Result<(), RendererError> {
    let (Some(output), Some(wanted)) = (read_output(shader), read_channels(format)) else {
        return Ok(());
    };
    let fits = output.scalar.kind == wanted.scalar.kind
        && output.scalar.width >= wanted.scalar.width
        && output.count >= wanted.count;
    if fits {
        return Ok(());
    }
    Err(PassError::InvalidColorTarget(format!(
        "pass '{}' draws into {format:?}, which takes {wanted}, and a shader writes {output} at @location(0)",
        pass.name
    ))
    .into())
}

/// The type the fragment entry point writes at `@location(0)`,
/// or `None` when it writes nothing there.
fn read_output(shader: &ShaderObject) -> Option<Channels> {
    let entry = shader.find_entry_point(naga::ShaderStage::Fragment)?;
    let result = entry.function.result.as_ref()?;
    let types = &shader.module.types;
    let first = |binding: &Option<naga::Binding>| {
        matches!(binding, Some(naga::Binding::Location { location: 0, .. }))
    };
    let ty = match (&result.binding, &types[result.ty].inner) {
        (binding, _) if first(binding) => result.ty,
        (None, naga::TypeInner::Struct { members, .. }) => {
            members.iter().find(|member| first(&member.binding))?.ty
        }
        _ => return None,
    };
    match types[ty].inner {
        naga::TypeInner::Scalar(scalar) => Some(Channels { scalar, count: 1 }),
        naga::TypeInner::Vector { size, scalar } => Some(Channels {
            scalar,
            count: size as u8,
        }),
        _ => None,
    }
}

/// The values a color `format` takes, the way wgpu reads them:
/// 32 bits per channel, or 64 for `R64Uint`.
fn read_channels(format: wgpu::TextureFormat) -> Option<Channels> {
    let kind = match format.sample_type(None, None)? {
        wgpu::TextureSampleType::Float { .. } => naga::ScalarKind::Float,
        wgpu::TextureSampleType::Uint => naga::ScalarKind::Uint,
        wgpu::TextureSampleType::Sint => naga::ScalarKind::Sint,
        wgpu::TextureSampleType::Depth => return None,
    };
    let width = match format {
        wgpu::TextureFormat::R64Uint => 8,
        _ => 4,
    };
    Some(Channels {
        scalar: naga::Scalar { kind, width },
        count: format.components(),
    })
}
