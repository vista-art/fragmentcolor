//! The checks a render runs on the CPU before it records a pass.
//!
//! Each rule here is one wgpu enforces too. Checking it first returns a typed
//! error from the call that broke it, on the web as well, where WebGPU reports
//! its own errors only after `render` has returned.

use super::access::{Access, Accesses, Writer};
use super::{PassDraws, RenderContext, RendererError};
use crate::material::AlphaMode;
use crate::pass::PassError;
use crate::texture::{Registration, read_format_features};
use crate::{
    PassObject, Renderable, ShaderObject, Target, TargetFrame, TextureObject, UniformData,
};
use parking_lot::RwLock;
use std::sync::Arc;

/// Refuses `item`, which needs `value` of the device's `limit`, when the device has less.
pub(crate) fn check_limit(
    item: &str,
    limit: &str,
    value: u64,
    max: u64,
) -> Result<(), RendererError> {
    match value > max {
        true => Err(RendererError::LimitError {
            item: item.to_string(),
            limit: limit.to_string(),
            value,
            max,
        }),
        false => Ok(()),
    }
}

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

/// Refuses a render that uses what another Renderer created, before it loads anything.
///
/// `context` is the rendering Renderer's, or `None` before it has one. A Renderer with no
/// context created no target or texture, so it refuses every render: `ForeignTarget` when a
/// render pass draws into a target that has an owner, else `NoContext`. With a context, a frame
/// target from another Renderer is `ForeignTarget`, unless every pass is a compute pass, and a
/// pass target or bound texture that this context doesn't hold is `TextureNotFoundError`.
/// The passes come from `peek_passes`, so a refusal leaves the renderable as it was.
pub(super) fn check_target(
    context: Option<&Arc<RenderContext>>,
    renderable: &impl Renderable,
    target: &impl Target,
) -> Result<(), RendererError> {
    let owner = target.owner();
    let foreign = match (context, &owner) {
        (Some(context), Some(owner)) => !Arc::ptr_eq(context, owner),
        _ => false,
    };
    let passes = renderable.peek_passes();
    let draws = passes.iter().any(|pass| !pass.is_compute());
    let Some(context) = context else {
        return Err(match draws && owner.is_some() {
            true => RendererError::ForeignTarget,
            false => RendererError::NoContext,
        });
    };
    if foreign && draws {
        return Err(RendererError::ForeignTarget);
    }
    for pass in passes.iter() {
        check_held(context, pass)?;
    }
    Ok(())
}

/// Refuses a texture `pass` draws into or binds that `context` doesn't hold.
/// `find_targets` and `check_textures` refuse the same in the plan, after `load` has run.
fn check_held(context: &RenderContext, pass: &PassObject) -> Result<(), RendererError> {
    let refuse = |registration: &Registration| match registration.belongs_to(context) {
        true => Ok(()),
        false => Err(RendererError::TextureNotFoundError(*registration.id())),
    };
    if !pass.is_compute() {
        for target in [&pass.color_target, &pass.depth_target] {
            if let Some(registration) = target.read().as_deref() {
                refuse(registration)?;
            }
        }
    }
    for shader in pass.shaders.read().iter() {
        for binding in shader.bound_textures.iter() {
            refuse(binding.value())?;
        }
    }
    Ok(())
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
        let limit = self.limits.max_compute_workgroups_per_dimension;
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

    /// Refuses a viewport with a side past the device's 2D texture limit, the largest
    /// viewport wgpu takes. A region within it keeps its size, past the target too.
    pub(super) fn check_viewport(&self, pass: &PassObject) -> Result<(), RendererError> {
        let limit = self.limits.max_texture_dimension_2d;
        match *pass.viewport.read() {
            Some(region) if region.width() > limit || region.height() > limit => {
                Err(PassError::ViewportLimit {
                    pass: pass.name.to_string(),
                    size: [region.width(), region.height()],
                    limit,
                }
                .into())
            }
            _ => Ok(()),
        }
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
                self.check_blend(pass, &plan.shader, color.format)?;
            }
            self.add_bindings(&mut accesses, &plan.shader)?;
        }
        Ok(())
    }

    /// Refuses `AlphaMode::Blend` into a `format` this device can't blend,
    /// and from a `shader` whose `@location(0)` output has no alpha channel.
    ///
    /// `AlphaMode::Blend` uses `BlendState::ALPHA_BLENDING`, whose factors read the source alpha.
    /// The WebGPU spec ("validating GPUFragmentState") refuses a blend factor that reads
    /// the source alpha when the output isn't a `vec4`. wgpu-core doesn't check that rule,
    /// and the browser does, so the check runs everywhere.
    fn check_blend(
        &self,
        pass: &PassObject,
        shader: &ShaderObject,
        format: wgpu::TextureFormat,
    ) -> Result<(), RendererError> {
        let blendable = wgpu::TextureFormatFeatureFlags::BLENDABLE;
        let adapter = self.adapter.as_ref();
        let flags = read_format_features(adapter, self.features, format).flags;
        if !flags.contains(blendable) {
            return Err(refuse_blend(pass, format));
        }
        match read_output(shader) {
            Some(output) if output.count < 4 => Err(PassError::InvalidColorTarget(format!(
                "pass '{}' blends a shader that writes {output} at @location(0), and AlphaMode::Blend reads an alpha channel; write a vec4",
                pass.name
            ))
            .into()),
            _ => Ok(()),
        }
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

/// The refusal of `AlphaMode::Blend` into a `format` the device can't blend. A format some
/// device blends, such as `Rgba32Float`, needs a feature this one lacks. One that no device
/// blends, such as an integer format, can't be the target of a blend at all.
fn refuse_blend(pass: &PassObject, format: wgpu::TextureFormat) -> RendererError {
    let blendable = wgpu::TextureFormatFeatureFlags::BLENDABLE;
    let anywhere = format
        .guaranteed_format_features(wgpu::Features::all())
        .flags;
    match anywhere.contains(blendable) {
        true => RendererError::FeatureError {
            item: format!("AlphaMode::Blend in pass '{}'", pass.name),
            feature: format!("blending into {format:?}"),
        },
        false => PassError::InvalidColorTarget(format!(
            "pass '{}' draws into {format:?}, which never blends, and a shader uses AlphaMode::Blend",
            pass.name
        ))
        .into(),
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

/// The color writes of the pipeline that runs `shader`: every channel, or none
/// when its fragment entry point writes nothing at `@location(0)`.
///
/// The WebGPU spec ("validating GPUFragmentState") takes a color target with no
/// fragment output at its location only with a `writeMask` of 0. wgpu-core accepts
/// any mask there, and the browser refuses the pipeline.
pub(super) fn pick_color_writes(shader: &ShaderObject) -> wgpu::ColorWrites {
    match find_output(shader) {
        Some(_) => wgpu::ColorWrites::ALL,
        None => wgpu::ColorWrites::empty(),
    }
}

/// The type of what the fragment entry point writes at `@location(0)`,
/// or `None` when it writes nothing there.
fn find_output(shader: &ShaderObject) -> Option<naga::Handle<naga::Type>> {
    let entry = shader.find_entry_point(naga::ShaderStage::Fragment)?;
    let result = entry.function.result.as_ref()?;
    let first = |binding: &Option<naga::Binding>| {
        matches!(binding, Some(naga::Binding::Location { location: 0, .. }))
    };
    match (&result.binding, &shader.module.types[result.ty].inner) {
        (binding, _) if first(binding) => Some(result.ty),
        (None, naga::TypeInner::Struct { members, .. }) => members
            .iter()
            .find(|member| first(&member.binding))
            .map(|member| member.ty),
        _ => None,
    }
}

/// The scalar type and channel count the fragment entry point writes at `@location(0)`,
/// or `None` when it writes nothing there.
fn read_output(shader: &ShaderObject) -> Option<Channels> {
    let ty = find_output(shader)?;
    match shader.module.types[ty].inner {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::platform::all::{create_instance, request_adapter};
    use crate::{PassType, Shader};

    /// A fullscreen triangle that samples `source`.
    const SAMPLE: &str = r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var soft: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(source, soft, position.xy / 8.0);
}
"#;

    /// A context on a device that opens with exactly `features`,
    /// or `None` when the adapter doesn't offer them.
    async fn open(features: wgpu::Features) -> Option<RenderContext> {
        let instance = create_instance().await;
        let adapter = request_adapter(&instance, None).await.ok()?;
        if !adapter.features().contains(features) {
            return None;
        }
        let descriptor = wgpu::DeviceDescriptor {
            required_features: features,
            required_limits: adapter.limits(),
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&descriptor).await.ok()?;
        let mut context = RenderContext::new(device, queue);
        context.adapter = Some(adapter);
        Some(context)
    }

    /// Whether `context` blends into Rgba32Float by wgpu's rule: a backend below WebGPU,
    /// such as GL with color_buffer_float, reports its own table, and every other one
    /// blends it when the device opened with FLOAT32_BLENDABLE.
    fn blends_float32(context: &RenderContext) -> bool {
        let adapter = context.adapter.as_ref().expect("an adapter");
        let flags = adapter.get_downlevel_capabilities().flags;
        match flags.contains(wgpu::DownlevelFlags::WEBGPU_TEXTURE_FORMAT_SUPPORT) {
            true => context.features.contains(wgpu::Features::FLOAT32_BLENDABLE),
            false => adapter
                .get_texture_format_features(wgpu::TextureFormat::Rgba32Float)
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::BLENDABLE),
        }
    }

    // Story: AlphaMode::Blend into Rgba32Float blends on a device that blends the format
    // and returns FeatureError on one that doesn't, on a device opened with float32
    // blending and on one opened without it.
    #[test]
    fn rgba32float_blends_where_the_device_can() {
        pollster::block_on(async move {
            let pass = PassObject::new("glass", PassType::Render);
            let shader = Shader::default();
            let float = wgpu::TextureFormat::Rgba32Float;
            for features in [wgpu::Features::empty(), wgpu::Features::FLOAT32_BLENDABLE] {
                let Some(context) = open(features).await else {
                    eprintln!("the adapter offers no {features:?}");
                    continue;
                };
                let result = context.check_blend(&pass, &shader.object, float);
                match blends_float32(&context) {
                    true => assert!(result.is_ok(), "{features:?}: {result:?}"),
                    false => assert!(
                        matches!(result, Err(RendererError::FeatureError { .. })),
                        "{features:?}: {result:?}"
                    ),
                }
            }
        });
    }

    // Story: a Renderer opens its device with FLOAT32_BLENDABLE whenever the adapter offers it.
    #[test]
    fn the_device_takes_float32_blending_when_offered() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let context = renderer.context(None).await.expect("a context");
            let adapter = context.adapter.as_ref().expect("an adapter");
            let blending = wgpu::Features::FLOAT32_BLENDABLE;
            assert_eq!(
                context.device.features().contains(blending),
                adapter.features().contains(blending)
            );
        });
    }

    // Story: a pipeline writes every color channel when its shader writes `@location(0)`,
    // directly or through a struct, and none when it writes no color.
    #[test]
    fn color_writes_follow_the_fragment_output() {
        let vertex =
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }";
        let pick = |fragment: &str| {
            let source = format!("{vertex}\n{fragment}");
            let shader = Shader::new(source.as_str()).expect("a shader");
            pick_color_writes(&shader.object)
        };
        let direct =
            pick("@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }");
        let member = pick(
            "struct Out { @location(0) color: vec4<f32> };\n@fragment fn fs_main() -> Out { return Out(vec4<f32>(1.0)); }",
        );
        let none = pick("@fragment fn fs_main() {}");
        assert_eq!(direct, wgpu::ColorWrites::ALL);
        assert_eq!(member, wgpu::ColorWrites::ALL);
        assert_eq!(none, wgpu::ColorWrites::empty());
    }

    // Story: a texture binding that waits in the shader's queue, because a render held
    // its storage, is applied before the checks, so binding the render target is refused.
    #[test]
    fn a_queued_binding_is_checked() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let frame = renderer
                .create_texture_target([8, 8])
                .await
                .expect("a target");
            let shader = Shader::new(SAMPLE).expect("a shader");
            {
                let _busy = shader.object.storage.read();
                shader
                    .set("source", &frame.texture())
                    .expect("a queued set");
            }
            assert!(
                !shader.object.pending.is_empty(),
                "the set was applied at once"
            );

            let result = renderer.render(&shader, &frame);
            assert!(
                matches!(
                    result,
                    Err(RendererError::PassError(PassError::TextureConflict { .. }))
                ),
                "{result:?}"
            );
        });
    }

    /// A Model that covers the view, drawn with `material`.
    fn build_model(material: crate::Material) -> crate::Model {
        let mesh = crate::Mesh::new();
        let corners = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
        mesh.add_vertices(
            corners.map(|corner| {
                crate::Vertex::pbr(corner).set(crate::Vertex::NORMAL, [0.0, 0.0, 1.0])
            }),
        );
        crate::Model::new(mesh, material)
    }

    /// A pass that draws `material`, whose base color map waits for an upload, and its shader.
    fn build_pass(material: crate::Material) -> (crate::Pass, Shader) {
        let material = material.base_color_texture((vec![255; 16], [2, 2]));
        let shader = material.shader.clone();
        let pass = crate::Pass::new("queued map");
        pass.add(&build_model(material)).expect("add");
        (pass, shader)
    }

    /// The texture id the shader binds to `key`.
    fn read_bound(shader: &Shader, key: &str) -> crate::texture::TextureId {
        match shader.object.get_uniform(key).map(|uniform| uniform.data) {
            Ok(UniformData::Texture(meta)) => meta.id,
            other => panic!("{key} holds no texture: {other:?}"),
        }
    }

    /// Checks that `shader`'s base color map waits in its queue, uploaded nowhere.
    fn check_queued(shader: &Shader) {
        let queued = shader.object.pending_textures.read().len();
        assert_eq!(queued, 1, "the map left the queue");
    }

    /// Checks that `renderer` drew `shader`'s base color map from its own context.
    fn check_drawn(renderer: &crate::Renderer, shader: &Shader) {
        let context = renderer.context.read().clone().expect("a context");
        let id = read_bound(shader, "base_color_map");
        assert!(
            context.get_texture(&id).is_some(),
            "the map isn't the owner's"
        );
    }

    /// A target with no owner yet, like Python's WindowTarget before it has a surface.
    struct Ownerless;

    impl crate::target::FrameSource for Ownerless {
        fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
            Err(crate::SurfaceError::Lost)
        }

        fn owner(&self) -> Option<Arc<RenderContext>> {
            None
        }
    }

    impl Target for Ownerless {
        fn size(&self) -> crate::Size {
            crate::Size::from((1u32, 1u32))
        }

        fn resize(
            &mut self,
            _size: impl Into<crate::Size>,
        ) -> Result<(), crate::texture::TextureError> {
            Ok(())
        }

        async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
            Ok(Vec::new())
        }
    }

    // Story: a render refused for a foreign target uploads nothing, so the map a
    // Material queued stays queued, and the owner's render uploads it into its own context.
    #[test]
    fn a_refused_render_leaves_a_queued_map_to_the_owner() {
        pollster::block_on(async move {
            let owner = crate::Renderer::new();
            let frame = owner.create_texture_target([8, 8]).await.expect("a target");
            let other = crate::Renderer::new();
            let _context = other.create_texture_target([8, 8]).await.expect("a target");
            let (pass, shader) = build_pass(crate::Material::pbr());

            let refused = other.render(&pass, &frame);
            assert!(
                matches!(refused, Err(RendererError::ForeignTarget)),
                "{refused:?}"
            );
            check_queued(&shader);
            owner
                .render(&pass, &frame)
                .expect("the owner draws the pass");
            check_drawn(&owner, &shader);
        });
    }

    // Story: a pass whose own target another Renderer created is refused before load,
    // so the Material's queued map stays queued for the Renderer that draws the pass.
    #[test]
    fn a_foreign_pass_target_leaves_a_queued_map() {
        pollster::block_on(async move {
            let owner = crate::Renderer::new();
            let frame = owner.create_texture_target([8, 8]).await.expect("a target");
            let offscreen = owner.create_texture_target([8, 8]).await.expect("a target");
            let other = crate::Renderer::new();
            let other_frame = other.create_texture_target([8, 8]).await.expect("a target");
            let (pass, shader) = build_pass(crate::Material::pbr());
            pass.set_target(&offscreen).expect("set_target");

            let refused = other.render(&pass, &other_frame);
            let id = *offscreen.texture().id();
            assert!(
                matches!(refused, Err(RendererError::TextureNotFoundError(found)) if found == id),
                "{refused:?}"
            );
            check_queued(&shader);
            owner
                .render(&pass, &frame)
                .expect("the owner draws the pass");
            check_drawn(&owner, &shader);
        });
    }

    // Story: a texture uniform bound to another Renderer's texture is refused before
    // load, so the Material's queued map stays queued for the Renderer that draws it.
    #[test]
    fn a_foreign_texture_leaves_a_queued_map() {
        pollster::block_on(async move {
            let owner = crate::Renderer::new();
            let frame = owner.create_texture_target([8, 8]).await.expect("a target");
            let normal = owner
                .create_texture((vec![128; 16], [2, 2]))
                .await
                .expect("a texture");
            let other = crate::Renderer::new();
            let other_frame = other.create_texture_target([8, 8]).await.expect("a target");
            let (pass, shader) = build_pass(crate::Material::pbr().normal_texture(&normal));

            let refused = other.render(&pass, &other_frame);
            assert!(
                matches!(refused, Err(RendererError::TextureNotFoundError(found)) if found == *normal.id()),
                "{refused:?}"
            );
            check_queued(&shader);
            owner
                .render(&pass, &frame)
                .expect("the owner draws the pass");
            check_drawn(&owner, &shader);
        });
    }

    // Story: a Scene refused for a foreign target gets no default camera or light, so a
    // light added afterwards is the only one the owner draws with.
    #[test]
    fn a_refused_scene_render_adds_no_defaults() {
        pollster::block_on(async move {
            let owner = crate::Renderer::new();
            let frame = owner.create_texture_target([8, 8]).await.expect("a target");
            let other = crate::Renderer::new();
            let _context = other.create_texture_target([8, 8]).await.expect("a target");
            let scene = crate::Scene::new();
            scene
                .add(&build_model(crate::Material::pbr()))
                .expect("add");

            let refused = other.render(&scene, &frame);
            assert!(
                matches!(refused, Err(RendererError::ForeignTarget)),
                "{refused:?}"
            );
            assert!(scene.cameras().is_empty(), "the refusal added a camera");
            assert!(scene.lights().is_empty(), "the refusal added a light");
        });
    }

    // Story: a Renderer with no context refuses another Renderer's target before it
    // opens a device, so the Material's queued map isn't uploaded anywhere.
    #[test]
    fn a_renderer_without_a_context_opens_no_device() {
        pollster::block_on(async move {
            let owner = crate::Renderer::new();
            let frame = owner.create_texture_target([8, 8]).await.expect("a target");
            let (pass, shader) = build_pass(crate::Material::pbr());
            let other = crate::Renderer::new();

            let refused = other.render(&pass, &frame);
            assert!(
                matches!(refused, Err(RendererError::ForeignTarget)),
                "{refused:?}"
            );
            assert!(
                other.context.read().is_none(),
                "the refusal opened a device"
            );
            check_queued(&shader);
        });
    }

    // Story: a Renderer with no context refuses a target with no owner, such as a
    // window without a surface, with NoContext, before it opens a device.
    #[test]
    fn a_renderer_without_a_context_refuses_an_ownerless_target() {
        let (pass, shader) = build_pass(crate::Material::pbr());
        let renderer = crate::Renderer::new();

        let refused = renderer.render(&pass, &Ownerless);
        assert!(
            matches!(refused, Err(RendererError::NoContext)),
            "{refused:?}"
        );
        assert!(
            renderer.context.read().is_none(),
            "the refusal opened a device"
        );
        check_queued(&shader);
    }
}
