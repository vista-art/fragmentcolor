//! The bind group layouts of a shader's pipeline, decided on the CPU before any wgpu call.
//!
//! The device checks and the pipeline builds read the same `Layout`, so what a
//! check passes is what the pipeline binds.

use super::lights::{self, LightPath};
use super::{PushMode, RenderContext};
use crate::shader::{PushEntry, StorageEntry, Uniform, UniformData};
use crate::shader::{
    list_explicit, lower_immediates, name_entries, validate_module, write_layouts,
};
use crate::texture::{TextureClass, TextureDim, TextureMeta, TextureScalarKind};
use crate::{ShaderError, ShaderObject};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroU64;
use std::sync::atomic::Ordering;

/// Bind group layout entries by group index, each group sorted by binding.
pub(super) type BindGroupEntries = BTreeMap<u32, Vec<wgpu::BindGroupLayoutEntry>>;

/// One layout entry and the root it binds, so a check can name the root.
#[derive(Debug, Clone)]
pub(super) struct Slot {
    pub(super) name: String,
    pub(super) group: u32,
    pub(super) entry: wgpu::BindGroupLayoutEntry,
}

/// What a pipeline binds on one device: every slot, and where the immediate roots go.
#[derive(Debug)]
pub(super) struct Layout {
    pub(super) slots: Vec<Slot>,
    pub(super) push: Option<PushMode>,
}

impl Layout {
    /// The slots by group, each group sorted by binding.
    pub(super) fn entries(&self) -> BindGroupEntries {
        let mut entries = BindGroupEntries::new();
        for slot in &self.slots {
            entries.entry(slot.group).or_default().push(slot.entry);
        }
        for list in entries.values_mut() {
            list.sort_by_key(|entry| entry.binding);
        }
        entries
    }
}

/// Lays out `shader` on the device of `context`. A render pipeline passes the device's
/// `lights` path, which adds the light roots, and a compute pipeline passes `None`.
pub(super) fn plan_layout(
    shader: &ShaderObject,
    context: &RenderContext,
    lights: Option<LightPath>,
) -> Layout {
    let mut slots = reflect_slots(shader);
    if let (Some(path), Some(roots)) = (lights, shader.light_roots.as_ref()) {
        let names = std::iter::once("lights").chain(roots.grid.map(|_| "light_grid"));
        for (name, (group, entry)) in names.zip(lights::layout_entries(roots, path)) {
            slots.push(Slot {
                name: name.to_string(),
                group,
                entry,
            });
        }
    }
    let push = plan_push(shader, context, &mut slots);
    Layout { slots, push }
}

/// The slots of the shader's reflected bindings, immediate roots left out.
fn reflect_slots(shader: &ShaderObject) -> Vec<Slot> {
    let storage = shader.storage.read();
    let mut slots = Vec::new();
    for (_, size, uniform) in storage.uniforms.values() {
        if uniform.name.contains('.') {
            continue;
        }
        let Some(ty) = binding_type(&uniform.data, *size) else {
            continue;
        };
        slots.push(Slot {
            name: uniform.name.clone(),
            group: uniform.group,
            entry: wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: pick_visibility(uniform),
                ty,
                count: None,
            },
        });
    }
    slots
}

/// The stages `uniform` is visible to: the stages that use it. A global no entry point
/// uses is visible to the compute stage, which asks no downlevel flag or feature of a
/// binding. wgpu still counts it against the compute stage's limits in every pipeline.
fn pick_visibility(uniform: &Uniform) -> wgpu::ShaderStages {
    match uniform.stages.is_empty() {
        true => wgpu::ShaderStages::COMPUTE,
        false => uniform.stages,
    }
}

/// The binding type a reflected root of `size` bytes takes, or `None` for an immediate root.
fn binding_type(data: &UniformData, size: u32) -> Option<wgpu::BindingType> {
    let ty = match data {
        UniformData::PushConstant(_) => return None,
        UniformData::Texture(meta) => texture_binding(meta),
        UniformData::Sampler(info) => wgpu::BindingType::Sampler(match info.comparison {
            true => wgpu::SamplerBindingType::Comparison,
            false => wgpu::SamplerBindingType::Filtering,
        }),
        UniformData::Storage(entries) => match entries.first() {
            Some(StorageEntry { span, access, .. }) => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: access.is_readonly(),
                },
                has_dynamic_offset: false,
                // A span of 0 can't bind, so it binds the 16 bytes WebGPU takes at least.
                min_binding_size: NonZeroU64::new(match *span {
                    0 => 16,
                    span => u64::from(span),
                }),
            },
            None => uniform_buffer(size),
        },
        _ => uniform_buffer(size),
    };
    Some(ty)
}

/// A uniform buffer of `size` bytes, padded to the 16 bytes WebGPU binds at least.
fn uniform_buffer(size: u32) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: NonZeroU64::new(wgpu::util::align_to(u64::from(size), 16).max(16)),
    }
}

/// The binding type a texture slot declares in WGSL.
pub(super) fn texture_binding(meta: &TextureMeta) -> wgpu::BindingType {
    let view_dimension = match (meta.dim, meta.arrayed) {
        (TextureDim::D1, _) => wgpu::TextureViewDimension::D1,
        (TextureDim::D2, false) => wgpu::TextureViewDimension::D2,
        (TextureDim::D2, true) => wgpu::TextureViewDimension::D2Array,
        (TextureDim::D3, _) => wgpu::TextureViewDimension::D3,
        (TextureDim::Cube, false) => wgpu::TextureViewDimension::Cube,
        (TextureDim::Cube, true) => wgpu::TextureViewDimension::CubeArray,
    };
    match meta.class {
        TextureClass::Depth { multi } => wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension,
            multisampled: multi,
        },
        TextureClass::Sampled { kind, multi } => wgpu::BindingType::Texture {
            sample_type: match kind {
                TextureScalarKind::Sint => wgpu::TextureSampleType::Sint,
                TextureScalarKind::Uint => wgpu::TextureSampleType::Uint,
                // A multisampled texture is never filtered.
                TextureScalarKind::Float | TextureScalarKind::Bool => {
                    wgpu::TextureSampleType::Float {
                        filterable: meta.sampled && !multi,
                    }
                }
            },
            view_dimension,
            multisampled: multi,
        },
        TextureClass::Storage { format, access } => wgpu::BindingType::StorageTexture {
            access: match (access.atomic, access.load, access.store) {
                (true, _, _) => wgpu::StorageTextureAccess::Atomic,
                (false, true, true) => wgpu::StorageTextureAccess::ReadWrite,
                (false, false, true) => wgpu::StorageTextureAccess::WriteOnly,
                (false, _, false) => wgpu::StorageTextureAccess::ReadOnly,
            },
            format: format.to_wgpu(),
            view_dimension,
        },
        TextureClass::External() => wgpu::BindingType::ExternalTexture,
    }
}

/// Decides where the immediate roots of `shader` go on the device of `context`,
/// adding their slots in fallback.
///
/// One root that fits the device's immediates takes native immediates. On the web, without
/// the `IMMEDIATES` feature, with more than one root, with a root past `max_immediate_size`,
/// and on a backend that pads its uniform bindings, each root becomes a uniform in the
/// lowest group no other slot uses.
fn plan_push(
    shader: &ShaderObject,
    context: &RenderContext,
    slots: &mut Vec<Slot>,
) -> Option<PushMode> {
    let mut roots: Vec<(String, u32, wgpu::ShaderStages)> = shader
        .storage
        .read()
        .uniforms
        .values()
        .filter(|(_, _, uniform)| !uniform.name.contains('.'))
        .filter_map(|(_, _, uniform)| match &uniform.data {
            UniformData::PushConstant(entries) => {
                entries.first().map(|PushEntry { span, .. }| {
                    (uniform.name.clone(), *span, pick_visibility(uniform))
                })
            }
            _ => None,
        })
        .collect();
    roots.sort_by(|a, b| a.0.cmp(&b.0));
    let [first, ..] = roots.as_slice() else {
        return None;
    };
    if takes_native(context, &roots) {
        return Some(PushMode::Native {
            root: first.0.clone(),
            size: first.1,
        });
    }
    let group = (0..)
        .find(|group| slots.iter().all(|slot| slot.group != *group))
        .unwrap_or(0);
    let mut bindings = HashMap::new();
    for (binding, (name, span, stages)) in (0u32..).zip(roots) {
        slots.push(Slot {
            name: name.clone(),
            group,
            entry: wgpu::BindGroupLayoutEntry {
                binding,
                visibility: stages,
                ty: uniform_buffer(span),
                count: None,
            },
        });
        bindings.insert(name, binding);
    }
    Some(PushMode::Fallback { group, bindings })
}

/// Whether one immediate root fits the device's native immediates. The web never takes
/// them, and neither does a backend that pads its uniform bindings to 16 bytes, where
/// the padded module pads the immediate roots it lowers too.
fn takes_native(context: &RenderContext, roots: &[(String, u32, wgpu::ShaderStages)]) -> bool {
    if cfg!(wasm) || context.aligned_uniform_bindings.load(Ordering::Relaxed) {
        return false;
    }
    let [(_, span, _)] = roots else {
        return false;
    };
    context.features.contains(wgpu::Features::IMMEDIATES)
        && *span <= context.limits.max_immediate_size
}

/// The module a pipeline for `shader` compiles: the padded module when `pad_uniforms` is set,
/// the lights lowered to a block of `uniform_lights`, and the immediate roots lowered in fallback.
pub(super) fn build_module(
    shader: &ShaderObject,
    layout: &Layout,
    pad_uniforms: bool,
    uniform_lights: Option<u32>,
) -> Result<naga::Module, ShaderError> {
    let module = shader.module_for_device(pad_uniforms, uniform_lights)?;
    Ok(match &layout.push {
        Some(PushMode::Fallback { group, bindings }) => lower_immediates(&module, *group, bindings),
        _ => module.into_owned(),
    })
}

/// The source a pipeline hands wgpu, with the name and stage of each entry point in it.
pub(super) struct PipelineSource {
    pub(super) source: wgpu::ShaderSource<'static>,
    pub(super) entries: Vec<(naga::ShaderStage, String)>,
}

/// The source a pipeline hands wgpu for `module`. On the browser's WebGPU (`writes_wgsl`),
/// wgpu writes a module back to WGSL without `@size` and `@align`, so a module whose
/// buffer structs set their layout with them goes as WGSL that keeps the layout. There every
/// entry point takes the name naga's writer gives it, from this exact module, in either source.
pub(super) fn pick_source(
    module: naga::Module,
    writes_wgsl: bool,
) -> Result<PipelineSource, ShaderError> {
    if !writes_wgsl {
        let entries = module
            .entry_points
            .iter()
            .map(|entry| (entry.stage, entry.name.clone()))
            .collect();
        let source = wgpu::ShaderSource::Naga(Cow::Owned(module));
        return Ok(PipelineSource { source, entries });
    }
    let entries = name_entries(&module);
    // Validating costs a pass over the module, so only a module with explicit layouts pays it.
    if !list_explicit(&module)?.is_empty() {
        let info = validate_module(&module, None)?;
        if let Some(text) = write_layouts(&module, &info)? {
            let source = wgpu::ShaderSource::Wgsl(Cow::Owned(text));
            return Ok(PipelineSource { source, entries });
        }
    }
    let source = wgpu::ShaderSource::Naga(Cow::Owned(module));
    Ok(PipelineSource { source, entries })
}

/// The name the first entry point of `stage` takes in a pipeline's source.
pub(super) fn find_entry(
    entries: &[(naga::ShaderStage, String)],
    stage: naga::ShaderStage,
) -> Result<&str, ShaderError> {
    entries
        .iter()
        .find(|(entry, _)| *entry == stage)
        .map(|(_, name)| name.as_str())
        .ok_or_else(|| {
            ShaderError::ParseError(format!(
                "the shader has no {stage:?} entry point, which this pipeline runs"
            ))
        })
}

/// Builds one layout for each group in `entries`, and an empty one for each lower group
/// that has none, so a pipeline layout's slots match the shader's group numbers.
pub(super) fn create_bind_group_layouts(
    device: &wgpu::Device,
    entries: &BindGroupEntries,
) -> HashMap<u32, wgpu::BindGroupLayout> {
    let mut layouts = HashMap::new();
    let Some(last) = entries.keys().max().copied() else {
        return layouts;
    };
    for group in 0..=last {
        let list = entries.get(&group).map(Vec::as_slice).unwrap_or_default();
        let label = match list.is_empty() {
            true => "Empty Bind Group Layout (placeholder)".to_string(),
            false => format!("Bind Group Layout for group: {group}"),
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: list,
        });
        layouts.insert(group, layout);
    }
    layouts
}

/// The pipeline layout over `layouts` in group order, with `immediate_size` bytes of immediates.
pub(super) fn create_pipeline_layout(
    device: &wgpu::Device,
    label: &str,
    layouts: &HashMap<u32, wgpu::BindGroupLayout>,
    immediate_size: u32,
) -> wgpu::PipelineLayout {
    let mut groups: Vec<&u32> = layouts.keys().collect();
    groups.sort();
    let sorted: Vec<Option<&wgpu::BindGroupLayout>> =
        groups.into_iter().map(|group| layouts.get(group)).collect();
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &sorted,
        immediate_size,
    })
}

/// The bytes of native immediates a pipeline layout declares for `push`.
pub(super) fn read_immediate_size(push: Option<&PushMode>) -> u32 {
    match push {
        Some(PushMode::Native { size, .. }) => *size,
        _ => 0,
    }
}
