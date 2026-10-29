//! The checks a shader passes on the CPU before any wgpu call.
//!
//! Each runs before the frame is acquired, so every platform returns the same
//! typed error from the call that caused it. The web has no error scope that
//! resolves in time, so these checks are what keep it in step with native.

use super::layout::{Layout, Slot, build_module, plan_layout, texture_binding};
use super::lights::LightPath;
use super::{PbrDefaults, PushMode, RenderContext, RendererError, report_missing};
use crate::shader::error::describe;
use crate::shader::{list_explicit, write_layouts};
use crate::texture::format_features;
use crate::{ShaderObject, TextureObject, UniformData};
use naga::ShaderStage;
use naga::valid::ModuleInfo;
use std::sync::atomic::Ordering;
use wgpu::{BindingType, BufferBindingType, DownlevelFlags, ShaderStages, StorageTextureAccess};

/// What `check_shader` decided for a shader source on one device.
#[derive(Debug, Clone)]
pub(super) enum Verdict {
    Passed,
    Refused(Refusal),
}

/// A refusal `check_shader` keeps, so the next frame returns it without checking again.
#[derive(Debug, Clone)]
pub(super) enum Refusal {
    Limit {
        item: String,
        limit: String,
        value: u64,
        max: u64,
    },
    Feature {
        item: String,
        feature: String,
    },
    Entry {
        stage: String,
    },
}

impl From<Refusal> for RendererError {
    fn from(refusal: Refusal) -> Self {
        match refusal {
            Refusal::Limit {
                item,
                limit,
                value,
                max,
            } => RendererError::LimitError {
                item,
                limit,
                value,
                max,
            },
            Refusal::Feature { item, feature } => RendererError::FeatureError { item, feature },
            Refusal::Entry { stage } => RendererError::EntryError { stage },
        }
    }
}

/// The facts about a texture that decide which slots take it.
#[derive(Debug, Clone, Copy)]
pub(super) struct TextureFacts {
    pub(super) format: wgpu::TextureFormat,
    pub(super) usage: wgpu::TextureUsages,
    pub(super) dimension: wgpu::TextureViewDimension,
    pub(super) samples: u32,
    pub(super) levels: u32,
}

impl TextureFacts {
    /// The facts about `texture` and the view `TextureObject::create_view` makes of it.
    pub(super) fn read(texture: &TextureObject) -> Self {
        let dimension = match texture.inner.dimension() {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => match texture.inner.depth_or_array_layers() {
                1 => wgpu::TextureViewDimension::D2,
                _ => wgpu::TextureViewDimension::D2Array,
            },
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        };
        Self {
            format: texture.format(),
            usage: texture.usage,
            dimension,
            samples: texture.inner.sample_count(),
            levels: texture.inner.mip_level_count(),
        }
    }

    /// The 1x1 default an unset PBR map binds.
    const PBR_DEFAULT: Self = Self {
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST),
        dimension: wgpu::TextureViewDimension::D2,
        samples: 1,
        levels: 1,
    };
}

impl RenderContext {
    /// Refuses a shader this device can't run, before any GPU work: groups and bindings past
    /// the device's limits, too many bindings of a kind in one stage, buffers past their size
    /// limits, storage the device can't bind, missing entry points, a workgroup past the limits,
    /// explicit layouts the backend can't keep, and anything else the device's own shader
    /// validator refuses. The verdict is kept by source hash, so each source is checked once
    /// per context, whether it passes or not.
    pub(super) fn check_shader(&self, shader: &ShaderObject) -> Result<(), RendererError> {
        if let Some(verdict) = self.checked.get(&shader.hash) {
            return match verdict.value() {
                Verdict::Passed => Ok(()),
                Verdict::Refused(refusal) => Err(refusal.clone().into()),
            };
        }
        let verdict = match self.judge_shader(shader)? {
            Ok(()) => Verdict::Passed,
            Err(refusal) => Verdict::Refused(refusal),
        };
        self.checked.insert(shader.hash, verdict.clone());
        match verdict {
            Verdict::Passed => Ok(()),
            Verdict::Refused(refusal) => Err(refusal.into()),
        }
    }

    /// Checks `shader` against the device. The outer error is one no verdict keeps,
    /// such as a lowering that fails, and the inner one is the device's refusal.
    fn judge_shader(&self, shader: &ShaderObject) -> Result<Result<(), Refusal>, RendererError> {
        let compute = shader.is_compute();
        let lights = (!compute).then(|| self.light_path());
        let layout = plan_layout(shader, self, lights);
        let inputs = match compute {
            true => Vec::new(),
            false => shader.reflect_vertex_inputs()?,
        };
        let judged = || -> Result<(), Refusal> {
            for slot in &layout.slots {
                let name = describe_slot(&layout, slot);
                check_slot(&name, slot, &self.limits)?;
                self.check_storage(&name, slot)?;
            }
            check_counts(&layout.slots, &self.limits)?;
            if !compute {
                check_entries(shader)?;
            }
            // Inputs come sorted by location, each location once,
            // so the last one also keeps the count within the limit.
            if let Some(input) = inputs.last() {
                let item = format!(
                    "shader input '{}' @location({})",
                    input.name, input.location
                );
                let max = u64::from(self.limits.max_vertex_attributes);
                check_limit(
                    &item,
                    "max_vertex_attributes",
                    u64::from(input.location) + 1,
                    max,
                )?;
            }
            Ok(())
        };
        if let Err(refusal) = judged() {
            return Ok(Err(refusal));
        }
        let (module, info) = match self.check_module(shader, &layout, lights)? {
            Ok(checked) => checked,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if compute && let Err(refusal) = self.check_compute_stage(&module, &info) {
            return Ok(Err(refusal));
        }
        Ok(Ok(()))
    }

    /// Refuses a texture uniform with no texture, or with a texture its slot can't take.
    /// An unset PBR map in a render shader takes its 1x1 default, when its slot takes that.
    pub(super) fn check_textures(&self, shader: &ShaderObject) -> Result<(), RendererError> {
        let compute = shader.is_compute();
        let storage = shader.storage.read();
        for (_, _, uniform) in storage.uniforms.values() {
            let UniformData::Texture(meta) = &uniform.data else {
                continue;
            };
            if uniform.name.contains('.') {
                continue;
            }
            let slot = texture_binding(meta);
            match self.get_texture(&meta.id) {
                Some(texture) => {
                    let facts = TextureFacts::read(&texture);
                    check_texture(&uniform.name, &slot, &facts, self.features)?
                }
                // An unset PBR map takes its default, which a compute pass never binds.
                // One set to a texture this context doesn't hold, removed or from another
                // Renderer, is refused.
                None if !compute && meta.id.id == 0 && PbrDefaults::has_slot(&uniform.name) => {
                    let facts = TextureFacts::PBR_DEFAULT;
                    check_texture(&uniform.name, &slot, &facts, self.features)?
                }
                None => return Err(report_missing(&uniform.name, meta)),
            }
        }
        Ok(())
    }

    /// Refuses storage the device can't bind: a storage texture format or access it lacks,
    /// a storage buffer the vertex stage reads, and storage the fragment stage writes.
    pub(super) fn check_storage(&self, name: &str, slot: &Slot) -> Result<(), Refusal> {
        let visibility = slot.entry.visibility;
        let writes = match slot.entry.ty {
            BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only },
                ..
            } => {
                if visibility.contains(ShaderStages::VERTEX) {
                    require(name, self.downlevel, DownlevelFlags::VERTEX_STORAGE)?;
                }
                !read_only
            }
            BindingType::StorageTexture { access, format, .. } => {
                self.check_storage_format(name, format, access)?;
                access != StorageTextureAccess::ReadOnly
            }
            _ => return Ok(()),
        };
        if writes && visibility.contains(ShaderStages::FRAGMENT) {
            require(
                name,
                self.downlevel,
                DownlevelFlags::FRAGMENT_WRITABLE_STORAGE,
            )?;
        }
        Ok(())
    }

    /// Refuses a storage texture `format` the device can't bind with `access`.
    fn check_storage_format(
        &self,
        name: &str,
        format: wgpu::TextureFormat,
        access: StorageTextureAccess,
    ) -> Result<(), Refusal> {
        let missing = format.required_features() - self.features;
        if !missing.is_empty() {
            return Err(Refusal::Feature {
                item: name.to_string(),
                feature: format!("{missing} for a {format:?} storage texture"),
            });
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
            return Err(Refusal::Feature {
                item: name.to_string(),
                feature: format!("{access:?} storage access to {format:?}"),
            });
        }
        Ok(())
    }

    /// Refuses a module the device's own shader validator refuses, such as one that needs
    /// a downlevel flag the device lacks, and explicit layouts the backend can't keep.
    /// It checks the module the pipeline compiles, and returns it with its overrides at
    /// their defaults, as the pipeline sets them, and its validation.
    fn check_module(
        &self,
        shader: &ShaderObject,
        layout: &Layout,
        lights: Option<LightPath>,
    ) -> Result<Result<(naga::Module, ModuleInfo), Refusal>, RendererError> {
        let pad = self.aligned_uniform_bindings.load(Ordering::Relaxed);
        let uniform_lights = lights.and_then(|path| path.uniform_lights());
        // The shader's own module, before padding renames a padded struct's copy.
        if let Err(refusal) = self.check_layouts(&shader.module) {
            return Ok(Err(refusal));
        }
        if let Err(refusal) = check_measured(shader, uniform_lights) {
            return Ok(Err(refusal));
        }
        let module = build_module(shader, layout, pad, uniform_lights)?;
        let mut validator = wgpu_naga_bridge::create_validator(
            self.features,
            self.downlevel,
            naga::valid::ValidationFlags::all(),
        );
        let info = match validator.validate(&module) {
            Ok(info) => info,
            Err(error) => return Ok(Err(refuse_module(&error))),
        };
        if self.writes_wgsl.load(Ordering::Relaxed)
            && let Err(error) = write_layouts(&module, &info)
        {
            return Ok(Err(Refusal::Feature {
                item: "a struct that sets its layout with @size or @align".into(),
                feature: format!("WGSL that keeps that layout on the browser's WebGPU ({error})"),
            }));
        }
        let constants = naga::back::PipelineConstants::default();
        match naga::back::pipeline_constants::process_overrides(&module, &info, None, &constants) {
            Ok((resolved, info)) => Ok(Ok((resolved.into_owned(), info.into_owned()))),
            Err(error) => Ok(Err(refuse_module(&error))),
        }
    }

    /// Refuses on a GL backend a buffer struct whose layout `@size` or `@align` sets.
    /// naga writes such a struct into a std140 block member by member, at the offsets its
    /// member types give, so its explicit layout can't hold there.
    fn check_layouts(&self, module: &naga::Module) -> Result<(), Refusal> {
        if self.backend != wgpu::Backend::Gl {
            return Ok(());
        }
        match list_explicit(module)
            .ok()
            .and_then(|names| names.into_iter().next())
        {
            Some(name) => Err(Refusal::Feature {
                item: format!("struct `{name}` (spell its padding out as members instead)"),
                feature: "@size and @align offsets in a std140 block".into(),
            }),
            None => Ok(()),
        }
    }

    /// Refuses a compute shader on a device without compute, a workgroup past the limits,
    /// and more workgroup memory than the device holds. `module` has its overrides set,
    /// so an override that sizes a workgroup or its memory is checked too.
    fn check_compute_stage(&self, module: &naga::Module, info: &ModuleInfo) -> Result<(), Refusal> {
        require(
            "the compute shader",
            self.downlevel,
            DownlevelFlags::COMPUTE_SHADERS,
        )?;
        let compute = |(_, entry): &(usize, &naga::EntryPoint)| entry.stage == ShaderStage::Compute;
        let Some((index, entry)) = module.entry_points.iter().enumerate().find(compute) else {
            return Ok(());
        };
        let item = format!("`{}`", entry.name);
        let limits = &self.limits;
        let maxes = [
            (
                limits.max_compute_workgroup_size_x,
                "max_compute_workgroup_size_x",
            ),
            (
                limits.max_compute_workgroup_size_y,
                "max_compute_workgroup_size_y",
            ),
            (
                limits.max_compute_workgroup_size_z,
                "max_compute_workgroup_size_z",
            ),
        ];
        for (size, (max, limit)) in entry.workgroup_size.iter().zip(maxes) {
            check_limit(&item, limit, u64::from(*size), u64::from(max))?;
        }
        let invocations = entry
            .workgroup_size
            .iter()
            .map(|&size| u64::from(size))
            .product();
        let max = u64::from(limits.max_compute_invocations_per_workgroup);
        check_limit(
            &item,
            "max_compute_invocations_per_workgroup",
            invocations,
            max,
        )?;
        let memory = measure_workgroup(module, info.get_entry_point(index));
        let max = u64::from(limits.max_compute_workgroup_storage_size);
        check_limit(&item, "max_compute_workgroup_storage_size", memory, max)
    }
}

/// Refuses a shader that measures its light roots with `arrayLength` on a device that
/// takes lights through a uniform block of `uniform_lights`, where nothing can measure them.
fn check_measured(shader: &ShaderObject, uniform_lights: Option<u32>) -> Result<(), Refusal> {
    let measured = shader
        .light_roots
        .as_ref()
        .is_some_and(|roots| roots.is_measured());
    if uniform_lights.is_none() || !measured {
        return Ok(());
    }
    Err(Refusal::Feature {
        item: "a shader that measures `lights` or `light_grid` with `arrayLength` (loop to `lights.count` instead)".into(),
        feature: "clustered lights in storage buffers".into(),
    })
}

/// The bytes of workgroup memory the entry point `uses` reaches, each variable
/// rounded up to 16 bytes, as WebGPU counts them.
fn measure_workgroup(module: &naga::Module, uses: &naga::valid::FunctionInfo) -> u64 {
    let mut layouter = naga::proc::Layouter::default();
    if layouter.update(module.to_ctx()).is_err() {
        return 0;
    }
    module
        .global_variables
        .iter()
        .filter(|(handle, global)| {
            global.space == naga::AddressSpace::WorkGroup && !uses[*handle].is_empty()
        })
        .map(|(_, global)| wgpu::util::align_to(u64::from(layouter[global.ty].size), 16))
        .sum()
}

/// Refuses a value past the device's `max` for `limit`.
fn check_limit(item: &str, limit: &str, value: u64, max: u64) -> Result<(), Refusal> {
    if value <= max {
        return Ok(());
    }
    Err(Refusal::Limit {
        item: item.to_string(),
        limit: limit.to_string(),
        value,
        max,
    })
}

/// Refuses `item` on a device whose downlevel flags `downlevel` lack `flag`.
fn require(item: &str, downlevel: DownlevelFlags, flag: DownlevelFlags) -> Result<(), Refusal> {
    if downlevel.contains(flag) {
        return Ok(());
    }
    let names: Vec<&str> = flag.iter_names().map(|(name, _)| name).collect();
    Err(Refusal::Feature {
        item: item.to_string(),
        feature: names.join(" | "),
    })
}

/// The refusal of a module the device's shader validator refused with `error`.
fn refuse_module(error: &dyn std::error::Error) -> Refusal {
    Refusal::Feature {
        item: "the shader".into(),
        feature: format!(
            "what the device's shader validator refuses: {}",
            describe(error)
        ),
    }
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
                "immediate root '{}', bound as a uniform at @group({group}) @binding({binding}),",
                slot.name
            )
        }
        _ => format!("'{}' at @group({group}) @binding({binding})", slot.name),
    }
}

/// Refuses a slot past the device's groups, bindings or buffer sizes.
pub(super) fn check_slot(name: &str, slot: &Slot, limits: &wgpu::Limits) -> Result<(), Refusal> {
    let groups = u64::from(slot.group) + 1;
    check_limit(
        name,
        "max_bind_groups",
        groups,
        u64::from(limits.max_bind_groups),
    )?;
    let bindings = u64::from(slot.entry.binding) + 1;
    let max = u64::from(limits.max_bindings_per_bind_group);
    check_limit(name, "max_bindings_per_bind_group", bindings, max)?;
    let BindingType::Buffer {
        ty,
        min_binding_size: Some(size),
        ..
    } = slot.entry.ty
    else {
        return Ok(());
    };
    match ty {
        // The uniform pool binds each upload padded to the device's offset alignment.
        BufferBindingType::Uniform => {
            let align = u64::from(limits.min_uniform_buffer_offset_alignment.max(16));
            let bound = wgpu::util::align_to(size.get(), align);
            let max = limits.max_uniform_buffer_binding_size;
            check_limit(name, "max_uniform_buffer_binding_size", bound, max)
        }
        BufferBindingType::Storage { .. } => {
            let max = limits.max_storage_buffer_binding_size;
            check_limit(name, "max_storage_buffer_binding_size", size.get(), max)?;
            check_limit(name, "max_buffer_size", size.get(), limits.max_buffer_size)
        }
    }
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
    fn limit(self, limits: &wgpu::Limits) -> (u32, &'static str) {
        match self {
            Kind::UniformBuffer => (
                limits.max_uniform_buffers_per_shader_stage,
                "max_uniform_buffers_per_shader_stage",
            ),
            Kind::StorageBuffer => (
                limits.max_storage_buffers_per_shader_stage,
                "max_storage_buffers_per_shader_stage",
            ),
            Kind::SampledTexture => (
                limits.max_sampled_textures_per_shader_stage,
                "max_sampled_textures_per_shader_stage",
            ),
            Kind::Sampler => (
                limits.max_samplers_per_shader_stage,
                "max_samplers_per_shader_stage",
            ),
            Kind::StorageTexture => (
                limits.max_storage_textures_per_shader_stage,
                "max_storage_textures_per_shader_stage",
            ),
        }
    }
}

/// Refuses more bindings of one kind in a stage than the device takes, counting each binding
/// in every stage it is visible to, as wgpu does. Buffers of both kinds also count together.
fn check_counts(slots: &[Slot], limits: &wgpu::Limits) -> Result<(), Refusal> {
    let stages = [
        (ShaderStages::VERTEX, "the vertex stage"),
        (ShaderStages::FRAGMENT, "the fragment stage"),
        (ShaderStages::COMPUTE, "the compute stage"),
    ];
    let kinds = [
        Kind::UniformBuffer,
        Kind::StorageBuffer,
        Kind::SampledTexture,
        Kind::Sampler,
        Kind::StorageTexture,
    ];
    for (stage, item) in stages {
        let visible: Vec<Kind> = slots
            .iter()
            .filter(|slot| slot.entry.visibility.contains(stage))
            .filter_map(|slot| Kind::read(&slot.entry.ty))
            .collect();
        for kind in kinds {
            let count = visible.iter().filter(|seen| **seen == kind).count() as u64;
            let (max, limit) = kind.limit(limits);
            check_limit(item, limit, count, u64::from(max))?;
        }
        let buffers = visible
            .iter()
            .filter(|kind| matches!(kind, Kind::UniformBuffer | Kind::StorageBuffer))
            .count() as u64;
        let max = u64::from(limits.max_buffers_and_acceleration_structures_per_shader_stage);
        check_limit(
            item,
            "max_buffers_and_acceleration_structures_per_shader_stage",
            buffers,
            max,
        )?;
    }
    Ok(())
}

/// Refuses a render shader without the vertex and fragment entry points a render pass runs.
fn check_entries(shader: &ShaderObject) -> Result<(), Refusal> {
    for (stage, name) in [
        (ShaderStage::Vertex, "@vertex"),
        (ShaderStage::Fragment, "@fragment"),
    ] {
        if shader.find_entry_point(stage).is_none() {
            return Err(Refusal::Entry { stage: name.into() });
        }
    }
    Ok(())
}

/// Refuses a texture the slot `name` can't take, by the rules wgpu binds it with.
pub(super) fn check_texture(
    name: &str,
    slot: &BindingType,
    texture: &TextureFacts,
    features: wgpu::Features,
) -> Result<(), RendererError> {
    let wrong = |expected: String, found: String| RendererError::BindingError {
        uniform: name.to_string(),
        expected,
        found,
    };
    let TextureFacts {
        format,
        usage,
        dimension,
        samples,
        levels,
    } = *texture;
    match *slot {
        BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        } => {
            if !usage.contains(wgpu::TextureUsages::TEXTURE_BINDING) {
                let expected = "a texture with TEXTURE_BINDING usage".into();
                return Err(wrong(expected, format!("usage {usage:?}")));
            }
            check_view(name, view_dimension, dimension)?;
            if multisampled != (samples > 1) {
                let expected = match multisampled {
                    true => "a multisampled texture",
                    false => "a single-sampled texture",
                };
                return Err(wrong(expected.into(), format!("{samples} samples")));
            }
            let found = format.sample_type(None, Some(features));
            if !fits_sample_type(sample_type, found) {
                let found = match found {
                    Some(found) => format!("{format:?}, which samples as {found:?}"),
                    None => format!("{format:?}, which no texture slot samples"),
                };
                return Err(wrong(
                    format!("a texture that samples as {sample_type:?}"),
                    found,
                ));
            }
            Ok(())
        }
        BindingType::StorageTexture {
            format: declared,
            view_dimension,
            ..
        } => {
            if !usage.contains(wgpu::TextureUsages::STORAGE_BINDING) {
                let expected = "a texture with STORAGE_BINDING usage".into();
                return Err(wrong(expected, format!("usage {usage:?}")));
            }
            if format != declared {
                return Err(wrong(format!("{declared:?}"), format!("{format:?}")));
            }
            check_view(name, view_dimension, dimension)?;
            if levels != 1 {
                return Err(wrong(
                    "one mip level".into(),
                    format!("{levels} mip levels"),
                ));
            }
            Ok(())
        }
        BindingType::ExternalTexture => Err(wrong(
            "a video frame, which no texture fills yet".into(),
            "a texture".into(),
        )),
        _ => Ok(()),
    }
}

/// Refuses a texture whose view has another dimension than the slot. A FragmentColor
/// texture's view is a 1D, 2D or 3D view of all of it, so a cube or array slot takes none.
fn check_view(
    name: &str,
    slot: wgpu::TextureViewDimension,
    view: wgpu::TextureViewDimension,
) -> Result<(), RendererError> {
    if slot == view {
        return Ok(());
    }
    use wgpu::TextureViewDimension as View;
    let expected = match slot {
        View::Cube | View::CubeArray | View::D2Array => {
            format!("a {slot:?} view, which no FragmentColor texture makes yet")
        }
        _ => format!("a {slot:?} texture"),
    };
    Err(RendererError::BindingError {
        uniform: name.to_string(),
        expected,
        found: format!("a {view:?} texture"),
    })
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
