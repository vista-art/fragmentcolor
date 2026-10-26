//! The checks a shader passes on the CPU before any wgpu call.
//!
//! Each runs before the frame is acquired, so every platform returns the same
//! typed error from the call that caused it. The web has no error scope that
//! resolves in time, so these checks are what keep it in step with native.

use super::layout::{Layout, Slot, build_module, plan_layout, texture_binding};
use super::lights::LightPath;
use super::{PbrDefaults, PushMode, RenderContext, RendererError, report_missing};
use crate::shader::error::describe;
use crate::texture::format_features;
use crate::{ShaderError, ShaderObject, TextureObject, UniformData};
use naga::ShaderStage;
use std::sync::atomic::Ordering;
use wgpu::{BindingType, BufferBindingType, DownlevelFlags, ShaderStages, StorageTextureAccess};

impl RenderContext {
    /// Refuses a shader this device can't run, before any GPU work: groups and bindings past
    /// the device's limits, too many bindings of a kind in one stage, buffers past their size
    /// limits, storage the device can't bind, missing entry points, a workgroup past the limits,
    /// and anything else the device's own shader validator refuses. A shader that passes
    /// is remembered by its source hash, so each source is checked once per context.
    pub(super) fn check_shader(&self, shader: &ShaderObject) -> Result<(), RendererError> {
        if self.checked.contains_key(&shader.hash) {
            return Ok(());
        }
        let compute = shader.is_compute();
        let lights = (!compute).then(|| self.light_path());
        let layout = plan_layout(shader, &self.device, lights);
        let limits = self.device.limits();
        let downlevel = self.read_downlevel();
        for slot in &layout.slots {
            let name = describe_slot(&layout, slot);
            check_slot(&name, slot, &limits)?;
            self.check_storage(&name, slot, downlevel)?;
        }
        check_counts(&layout.slots, &limits)?;
        let module = self.check_module(shader, &layout, lights)?;
        match compute {
            true => check_compute(&module, &limits, downlevel)?,
            false => check_entries(shader)?,
        }
        self.checked.insert(shader.hash, ());
        Ok(())
    }

    /// Refuses a texture uniform with no texture, or with a texture its slot can't take.
    /// A PBR map slot with no texture takes its 1x1 default instead.
    pub(super) fn check_textures(&self, shader: &ShaderObject) -> Result<(), RendererError> {
        let storage = shader.storage.read();
        for (_, _, uniform) in storage.uniforms.values() {
            let UniformData::Texture(meta) = &uniform.data else {
                continue;
            };
            if uniform.name.contains('.') {
                continue;
            }
            match self.get_texture(&meta.id) {
                Some(texture) => check_texture(
                    &uniform.name,
                    &texture_binding(meta),
                    &texture,
                    self.device.features(),
                )?,
                None if PbrDefaults::has_slot(&uniform.name) => {}
                None => return Err(report_missing(&uniform.name, meta)),
            }
        }
        Ok(())
    }

    /// The downlevel flags of the context's adapter, or every flag when it has none.
    pub(super) fn read_downlevel(&self) -> DownlevelFlags {
        self.adapter
            .as_ref()
            .map(|adapter| adapter.get_downlevel_capabilities().flags)
            .unwrap_or(DownlevelFlags::all())
    }

    /// Refuses storage the device can't bind: a storage texture format or access it lacks,
    /// a storage buffer the vertex stage reads, and storage the fragment stage writes.
    fn check_storage(
        &self,
        name: &str,
        slot: &Slot,
        downlevel: DownlevelFlags,
    ) -> Result<(), RendererError> {
        let visibility = slot.entry.visibility;
        let writes = match slot.entry.ty {
            BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only },
                ..
            } => {
                if visibility.contains(ShaderStages::VERTEX)
                    && !downlevel.contains(DownlevelFlags::VERTEX_STORAGE)
                {
                    return Err(refuse(format!(
                        "{name} is a storage buffer the vertex stage reads, and this device has no storage in the vertex stage"
                    )));
                }
                !read_only
            }
            BindingType::StorageTexture { access, format, .. } => {
                self.check_storage_format(name, format, access)?;
                access != StorageTextureAccess::ReadOnly
            }
            _ => return Ok(()),
        };
        if writes
            && visibility.contains(ShaderStages::FRAGMENT)
            && !downlevel.contains(DownlevelFlags::FRAGMENT_WRITABLE_STORAGE)
        {
            return Err(refuse(format!(
                "{name} is storage the fragment stage writes, and this device takes no writes there"
            )));
        }
        Ok(())
    }

    /// Refuses a storage texture `format` the device can't bind with `access`.
    fn check_storage_format(
        &self,
        name: &str,
        format: wgpu::TextureFormat,
        access: StorageTextureAccess,
    ) -> Result<(), RendererError> {
        let missing = format.required_features() - self.device.features();
        if !missing.is_empty() {
            return Err(refuse(format!(
                "{name} is a {format:?} storage texture, which needs the device feature {missing:?}"
            )));
        }
        let features = format_features(self, format);
        let flag = match access {
            StorageTextureAccess::WriteOnly => wgpu::TextureFormatFeatureFlags::STORAGE_WRITE_ONLY,
            StorageTextureAccess::ReadOnly => wgpu::TextureFormatFeatureFlags::STORAGE_READ_ONLY,
            StorageTextureAccess::ReadWrite => wgpu::TextureFormatFeatureFlags::STORAGE_READ_WRITE,
            StorageTextureAccess::Atomic => wgpu::TextureFormatFeatureFlags::STORAGE_ATOMIC,
        };
        let binds = features
            .allowed_usages
            .contains(wgpu::TextureUsages::STORAGE_BINDING);
        if !binds || !features.flags.contains(flag) {
            return Err(refuse(format!(
                "{name} is a {format:?} storage texture with {access:?} access, which this device doesn't bind"
            )));
        }
        Ok(())
    }

    /// Refuses a module the device's own shader validator refuses, such as one that needs
    /// a downlevel flag the device lacks. It checks the module the pipeline compiles,
    /// and returns it with its overrides at their defaults, as the pipeline sets them.
    fn check_module(
        &self,
        shader: &ShaderObject,
        layout: &Layout,
        lights: Option<LightPath>,
    ) -> Result<naga::Module, RendererError> {
        let pad = lights.is_some() && self.aligned_uniform_bindings.load(Ordering::Relaxed);
        let uniform_lights = lights.and_then(|path| path.uniform_lights());
        let module = build_module(shader, layout, pad, uniform_lights)?;
        let mut validator = wgpu_naga_bridge::create_validator(
            self.device.features(),
            self.read_downlevel(),
            naga::valid::ValidationFlags::all(),
        );
        let info = validator
            .validate(&module)
            .map_err(|error| refuse_module(&error))?;
        let constants = naga::back::PipelineConstants::default();
        match naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants) {
            Ok((resolved, _)) => Ok(resolved.into_owned()),
            Err(error) => Err(refuse_module(&error)),
        }
    }
}

/// A refusal of what this device can't run.
fn refuse(message: String) -> RendererError {
    RendererError::Error(message)
}

/// The refusal of a module the device's shader validator refused with `error`.
fn refuse_module(error: &dyn std::error::Error) -> RendererError {
    refuse(format!(
        "this device can't run the shader: {}",
        describe(error)
    ))
}

/// How a message names `slot`: by its root, and as a lowered uniform for an immediate root.
fn describe_slot(layout: &Layout, slot: &Slot) -> String {
    let (group, binding) = (slot.group, slot.entry.binding);
    match &layout.push {
        Some(PushMode::Fallback {
            group: lowered,
            bindings,
        }) if *lowered == group && bindings.contains_key(&slot.name) => {
            format!(
                "immediate root '{}', which this device binds as a uniform at @group({group}) @binding({binding}),",
                slot.name
            )
        }
        _ => format!("'{}' at @group({group}) @binding({binding})", slot.name),
    }
}

/// Refuses a slot past the device's groups, bindings or buffer sizes.
pub(super) fn check_slot(
    name: &str,
    slot: &Slot,
    limits: &wgpu::Limits,
) -> Result<(), RendererError> {
    if slot.group >= limits.max_bind_groups {
        return Err(RendererError::BindGroupLayoutError(format!(
            "{name} is past the device's {} bind groups",
            limits.max_bind_groups
        )));
    }
    if slot.entry.binding >= limits.max_bindings_per_bind_group {
        return Err(RendererError::BindGroupLayoutError(format!(
            "{name} is past the device's {} bindings in a group",
            limits.max_bindings_per_bind_group
        )));
    }
    let BindingType::Buffer {
        ty,
        min_binding_size: Some(size),
        ..
    } = slot.entry.ty
    else {
        return Ok(());
    };
    let (bound, max, limit) = match ty {
        // The uniform pool binds each upload padded to the device's offset alignment.
        BufferBindingType::Uniform => {
            let align = u64::from(limits.min_uniform_buffer_offset_alignment.max(16));
            let bound = wgpu::util::align_to(size.get(), align);
            let max = limits.max_uniform_buffer_binding_size;
            (bound, max, "max_uniform_buffer_binding_size")
        }
        BufferBindingType::Storage { .. } => {
            let binding = limits.max_storage_buffer_binding_size;
            match binding <= limits.max_buffer_size {
                true => (size.get(), binding, "max_storage_buffer_binding_size"),
                false => (size.get(), limits.max_buffer_size, "max_buffer_size"),
            }
        }
    };
    if bound > max {
        return Err(RendererError::BindGroupLayoutError(format!(
            "{name} binds {bound} bytes, past the device's {limit} of {max}"
        )));
    }
    Ok(())
}

/// The kinds of binding a stage counts against its limits.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    UniformBuffer,
    StorageBuffer,
    SampledTexture,
    Sampler,
    StorageTexture,
}

impl Kind {
    /// The kind `ty` counts as, or `None` for a binding no stage limit counts.
    fn read(ty: &BindingType) -> Option<Self> {
        Some(match ty {
            BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                ..
            } => Kind::UniformBuffer,
            BindingType::Buffer { .. } => Kind::StorageBuffer,
            BindingType::Texture { .. } => Kind::SampledTexture,
            BindingType::Sampler(_) => Kind::Sampler,
            BindingType::StorageTexture { .. } => Kind::StorageTexture,
            _ => return None,
        })
    }

    /// The limit on this kind in one stage, and its name.
    fn limit(self, limits: &wgpu::Limits) -> (u32, &'static str, &'static str) {
        match self {
            Kind::UniformBuffer => (
                limits.max_uniform_buffers_per_shader_stage,
                "uniform buffers",
                "max_uniform_buffers_per_shader_stage",
            ),
            Kind::StorageBuffer => (
                limits.max_storage_buffers_per_shader_stage,
                "storage buffers",
                "max_storage_buffers_per_shader_stage",
            ),
            Kind::SampledTexture => (
                limits.max_sampled_textures_per_shader_stage,
                "sampled textures",
                "max_sampled_textures_per_shader_stage",
            ),
            Kind::Sampler => (
                limits.max_samplers_per_shader_stage,
                "samplers",
                "max_samplers_per_shader_stage",
            ),
            Kind::StorageTexture => (
                limits.max_storage_textures_per_shader_stage,
                "storage textures",
                "max_storage_textures_per_shader_stage",
            ),
        }
    }
}

/// Refuses more bindings of one kind in a stage than the device takes, counting each binding
/// in every stage it is visible to, as wgpu does. Buffers of both kinds also count together.
fn check_counts(slots: &[Slot], limits: &wgpu::Limits) -> Result<(), RendererError> {
    let stages = [
        (ShaderStages::VERTEX, "vertex"),
        (ShaderStages::FRAGMENT, "fragment"),
        (ShaderStages::COMPUTE, "compute"),
    ];
    let kinds = [
        Kind::UniformBuffer,
        Kind::StorageBuffer,
        Kind::SampledTexture,
        Kind::Sampler,
        Kind::StorageTexture,
    ];
    for (stage, stage_name) in stages {
        let visible: Vec<Kind> = slots
            .iter()
            .filter(|slot| slot.entry.visibility.contains(stage))
            .filter_map(|slot| Kind::read(&slot.entry.ty))
            .collect();
        for kind in kinds {
            let count = visible.iter().filter(|seen| **seen == kind).count();
            let (max, what, limit) = kind.limit(limits);
            if count > max as usize {
                return Err(RendererError::BindGroupLayoutError(format!(
                    "{count} {what} in the {stage_name} stage, past the device's {limit} of {max}"
                )));
            }
        }
        let buffers = visible
            .iter()
            .filter(|kind| matches!(kind, Kind::UniformBuffer | Kind::StorageBuffer))
            .count();
        let max = limits.max_buffers_and_acceleration_structures_per_shader_stage;
        if buffers > max as usize {
            return Err(RendererError::BindGroupLayoutError(format!(
                "{buffers} buffers in the {stage_name} stage, past the device's max_buffers_and_acceleration_structures_per_shader_stage of {max}"
            )));
        }
    }
    Ok(())
}

/// Refuses a render shader without the vertex and fragment entry points a render pass runs.
fn check_entries(shader: &ShaderObject) -> Result<(), RendererError> {
    for (stage, name) in [
        (ShaderStage::Vertex, "@vertex"),
        (ShaderStage::Fragment, "@fragment"),
    ] {
        if shader.find_entry_point(stage).is_none() {
            return Err(ShaderError::ParseError(format!(
                "the shader has no {name} entry point, which a render pass runs"
            ))
            .into());
        }
    }
    Ok(())
}

/// Refuses a compute shader on a device without compute, or with a workgroup past its limits.
/// `module` has its overrides set, so a workgroup an override sizes is checked too.
fn check_compute(
    module: &naga::Module,
    limits: &wgpu::Limits,
    downlevel: DownlevelFlags,
) -> Result<(), RendererError> {
    if !downlevel.contains(DownlevelFlags::COMPUTE_SHADERS) {
        return Err(refuse("this device runs no compute shaders".into()));
    }
    let compute = |entry: &&naga::EntryPoint| entry.stage == ShaderStage::Compute;
    let Some(entry) = module.entry_points.iter().find(compute) else {
        return Ok(());
    };
    let maxes = [
        limits.max_compute_workgroup_size_x,
        limits.max_compute_workgroup_size_y,
        limits.max_compute_workgroup_size_z,
    ];
    for ((size, max), axis) in entry.workgroup_size.iter().zip(maxes).zip(["x", "y", "z"]) {
        if *size > max {
            return Err(refuse(format!(
                "`{}` has a workgroup {size} wide on {axis}, past the device's max_compute_workgroup_size_{axis} of {max}",
                entry.name
            )));
        }
    }
    let invocations: u64 = entry
        .workgroup_size
        .iter()
        .map(|&size| u64::from(size))
        .product();
    let max = limits.max_compute_invocations_per_workgroup;
    if invocations > u64::from(max) {
        return Err(refuse(format!(
            "`{}` runs {invocations} invocations per workgroup, past the device's max_compute_invocations_per_workgroup of {max}",
            entry.name
        )));
    }
    Ok(())
}

/// Refuses a texture the slot `name` can't take, by the rules wgpu binds it with.
pub(super) fn check_texture(
    name: &str,
    slot: &BindingType,
    texture: &TextureObject,
    features: wgpu::Features,
) -> Result<(), RendererError> {
    let format = texture.format();
    let samples = texture.inner.sample_count();
    let dimension = read_view_dimension(texture);
    let wrong = |problem: String| {
        RendererError::BindGroupLayoutError(format!("texture '{name}' can't take {problem}"))
    };
    match *slot {
        BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        } => {
            if !texture.usage.contains(wgpu::TextureUsages::TEXTURE_BINDING) {
                return Err(wrong(format!(
                    "a texture without TEXTURE_BINDING usage ({:?})",
                    texture.usage
                )));
            }
            if dimension != view_dimension {
                return Err(wrong(format!(
                    "a {dimension:?} texture in a {view_dimension:?} slot"
                )));
            }
            if multisampled != (samples > 1) {
                return Err(wrong(format!(
                    "a texture with {samples} samples in a slot that is{} multisampled",
                    if multisampled { "" } else { " not" }
                )));
            }
            let found = format.sample_type(None, Some(features));
            if !fits_sample_type(sample_type, found) {
                return Err(wrong(format!(
                    "{format:?}, which samples as {found:?}, in a slot that samples as {sample_type:?}"
                )));
            }
            Ok(())
        }
        BindingType::StorageTexture {
            format: declared,
            view_dimension,
            ..
        } => {
            if !texture.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) {
                return Err(wrong(format!(
                    "a texture without STORAGE_BINDING usage ({:?})",
                    texture.usage
                )));
            }
            if format != declared {
                return Err(wrong(format!("{format:?} in a {declared:?} storage slot")));
            }
            if dimension != view_dimension {
                return Err(wrong(format!(
                    "a {dimension:?} texture in a {view_dimension:?} slot"
                )));
            }
            let levels = texture.inner.mip_level_count();
            if levels != 1 {
                return Err(wrong(format!(
                    "a texture with {levels} mip levels in a storage slot, which binds one"
                )));
            }
            Ok(())
        }
        BindingType::ExternalTexture => Err(wrong(
            "a texture in a texture_external slot, which no texture fills yet".into(),
        )),
        _ => Ok(()),
    }
}

/// Whether a texture that samples as `found` fills a slot that samples as `slot`.
/// An unfilterable float slot also takes filterable floats and depth.
fn fits_sample_type(slot: wgpu::TextureSampleType, found: Option<wgpu::TextureSampleType>) -> bool {
    use wgpu::TextureSampleType as Sample;
    matches!(
        (slot, found),
        (Sample::Uint, Some(Sample::Uint))
            | (Sample::Sint, Some(Sample::Sint))
            | (Sample::Depth, Some(Sample::Depth))
            | (
                Sample::Float { filterable: false },
                Some(Sample::Float { .. })
            )
            | (Sample::Float { filterable: false }, Some(Sample::Depth))
            | (
                Sample::Float { filterable: true },
                Some(Sample::Float { filterable: true })
            )
    )
}

/// The dimension of the view `TextureObject::create_view` makes, wgpu's default for the texture.
fn read_view_dimension(texture: &TextureObject) -> wgpu::TextureViewDimension {
    match texture.inner.dimension() {
        wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
        wgpu::TextureDimension::D2 => match texture.inner.depth_or_array_layers() {
            1 => wgpu::TextureViewDimension::D2,
            _ => wgpu::TextureViewDimension::D2Array,
        },
        wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
    }
}
