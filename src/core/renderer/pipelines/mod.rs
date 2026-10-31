//! The render and compute pipelines a renderer caches, and their keys.

pub(super) mod depth;
pub(super) mod layout;
pub(super) mod vertex_layouts;

use crate::core::renderer::lights::LightPath;
use crate::core::renderer::{RenderContext, RendererError, checks};
use crate::{AlphaMode, ShaderHash, ShaderObject};
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use dashmap::mapref::one::RefMut;
use depth::DepthUse;
use layout::{Layout, PipelineSource, build_module, create_bind_group_layouts};
use layout::{create_pipeline_layout, find_entry, measure_immediates, pick_source};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use vertex_layouts::VertexLayouts;

#[derive(Debug, Clone)]
pub(super) enum PushMode {
    // Use native push constants (only supported on native, single root for now)
    Native {
        root: String,
        size: u32,
    },
    // Fallback to uniforms (Web or size/multi-root on native)
    // Map variable name -> binding number within the fallback group
    Fallback {
        group: u32,
        bindings: HashMap<String, u32>,
    },
}

#[derive(Debug, Clone)]
pub(super) struct RenderPipeline {
    pub(super) pipeline: wgpu::RenderPipeline,
    // Map of bind group index -> layout (keeps group indices stable)
    pub(super) bind_group_layouts: HashMap<u32, wgpu::BindGroupLayout>,
    pub(super) push_mode: Option<PushMode>,
}

#[derive(Debug)]
pub(super) struct ComputePipeline {
    pub(super) pipeline: wgpu::ComputePipeline,
    pub(super) bind_group_layouts: HashMap<u32, wgpu::BindGroupLayout>,
    pub(super) push_mode: Option<PushMode>,
}

/// Key for caching render pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct RenderPipelineKey {
    pub(super) shader_hash: ShaderHash,
    pub(super) color_format: wgpu::TextureFormat,
    pub(super) depth: DepthUse,
    pub(super) sample_count: u32,
    /// `Material::alpha_mode` bakes into the pipeline: `Opaque`/`Mask`
    /// turn blending off, `Blend` flips on standard alpha-over and disables
    /// depth-write. Different modes against the same shader hash therefore
    /// need different pipelines.
    pub(super) alpha_mode: crate::AlphaMode,
    /// `Material::double_sided` flips the `cull_mode` on the primitive
    /// state, so it also baked into the pipeline.
    pub(super) double_sided: bool,
    /// Vertex buffer layouts of the shader's mesh, if any.
    /// Shaders that share a source can feed meshes with different strides,
    /// attribute offsets, or step modes, and each layout bakes into the pipeline.
    pub(super) vertex_layouts: Option<VertexLayouts>,
}

/// Key for caching compute pipelines
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ComputePipelineKey {
    pub(super) shader_hash: ShaderHash,
    /// The layout entries the pipeline's bind group layouts are built from.
    pub(super) bind_group_entries: layout::BindGroupEntries,
}

impl RenderContext {
    /// The pipeline cached under `key`, built by `build` inside error scopes on a miss.
    ///
    /// A pipeline that fails to build stays out of the cache, so the next frame builds
    /// it again and reports the same error. On wasm the error lands after the insert,
    /// so the pipeline leaves the cache then.
    pub(super) fn fetch_pipeline<'cache, K, V>(
        &self,
        cache: &'cache Arc<DashMap<K, V>>,
        key: K,
        label: &str,
        build: impl FnOnce() -> Result<V, RendererError>,
    ) -> Result<RefMut<'cache, K, V>, RendererError>
    where
        K: Eq + Hash + Clone + 'static,
        V: 'static,
    {
        match cache.entry(key) {
            Entry::Occupied(entry) => Ok(entry.into_ref()),
            Entry::Vacant(entry) => {
                let shared = Arc::downgrade(cache);
                let key = entry.key().clone();
                let evict = move || {
                    if let Some(cache) = shared.upgrade() {
                        cache.remove(&key);
                    }
                };
                let pipeline = self.validate_with(label, build, evict)?;
                Ok(entry.insert(pipeline))
            }
        }
    }
}

/// The override a shader declares to learn whether its target stores colors
/// encoded for display. `Material::pbr` encodes its linear light when it does.
const ENCODE_SRGB: &str = "ENCODE_SRGB";

/// Whether `format` stores colors encoded for display without encoding them
/// on write, which a shader that works in linear light has to do itself.
fn is_display(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
    )
}

/// `module` with its `ENCODE_SRGB` override set for a target of `format`.
/// FragmentColor resolves it here, because the WGSL that wgpu writes for the
/// browser from a naga module doesn't carry overrides. A module without that
/// override, or one whose overrides fail to resolve, comes back unchanged.
fn set_encoding(module: &naga::Module, format: wgpu::TextureFormat) -> naga::Module {
    let declared = module
        .overrides
        .iter()
        .any(|(_, item)| item.name.as_deref() == Some(ENCODE_SRGB));
    if !declared {
        return module.clone();
    }
    let flags = naga::valid::ValidationFlags::all();
    let info = match naga::valid::Validator::new(flags, naga::valid::Capabilities::all())
        .validate(module)
    {
        Ok(info) => info,
        Err(error) => {
            log::error!("{ENCODE_SRGB} stays at its default: {error}");
            return module.clone();
        }
    };
    let mut constants = naga::back::PipelineConstants::default();
    let encode = if is_display(format) { 1.0 } else { 0.0 };
    constants.insert(ENCODE_SRGB.to_string(), encode);
    match naga::back::pipeline_constants::process_overrides(module, &info, None, &constants) {
        Ok((resolved, _)) => resolved.into_owned(),
        Err(error) => {
            log::error!("{ENCODE_SRGB} stays at its default: {error}");
            module.clone()
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_render_pipeline(
    device: &wgpu::Device,
    shader: &ShaderObject,
    layout: &Layout,
    format: wgpu::TextureFormat,
    sample_count: u32,
    vertex_layouts: Option<&VertexLayouts>,
    depth: DepthUse,
    alpha_mode: crate::AlphaMode,
    double_sided: bool,
    pad_uniforms: bool,
    light_path: LightPath,
    writes_wgsl: bool,
) -> Result<RenderPipeline, RendererError> {
    // Every CPU check comes before the first wgpu call, so a refusal builds nothing.
    let module = build_module(shader, layout, pad_uniforms, light_path.uniform_lights())?;
    let PipelineSource { source, entries } =
        pick_source(set_encoding(&module, format), writes_wgsl)?;
    let vs_entry = find_entry(&entries, naga::ShaderStage::Vertex)?;
    let fs_entry = find_entry(&entries, naga::ShaderStage::Fragment)?;

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source,
    });
    let layouts = create_bind_group_layouts(device, &layout.list_entries());
    let immediate_size = measure_immediates(layout.push.as_ref());
    let pipeline_layout =
        create_pipeline_layout(device, "Default Pipeline Layout", &layouts, immediate_size);

    let vertex_buffers: Vec<Option<wgpu::VertexBufferLayout>> = vertex_layouts
        .map(VertexLayouts::buffers)
        .unwrap_or_default()
        .into_iter()
        .map(Some)
        .collect();

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Default Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some(vs_entry),
            buffers: &vertex_buffers,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some(fs_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                // Blend state baked from Material::alpha_mode:
                //   Opaque / Mask -> no blending (alpha bits go to the FB
                //                    but the equation ignores them; Mask
                //                    does its work via `discard` in the
                //                    fragment shader).
                //   Blend         -> standard SrcAlpha/OneMinusSrcAlpha
                //                    over-blend.
                blend: match alpha_mode {
                    AlphaMode::Opaque | AlphaMode::Mask => None,
                    AlphaMode::Blend => Some(wgpu::BlendState::ALPHA_BLENDING),
                },
                // Custom (non-glTF-MR) blend equations are tracked under
                // the roadmap's "Custom blending" item — Material's
                // AlphaMode covers the three glTF 2.0 modes; anything more
                // exotic will ride on a separate per-Material slot.
                write_mask: checks::pass::pick_writes(shader),
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            // `double_sided=true` (the ShaderObject default) leaves
            // `cull_mode: None`, matching the renderer's pre-Material
            // behavior. `Material::pbr` (and `Material::custom`) flip
            // it to `false`, which engages standard back-face culling —
            // the glTF 2.0 default for single-sided materials.
            cull_mode: if double_sided {
                None
            } else {
                Some(wgpu::Face::Back)
            },
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil: depth.describe_state(alpha_mode),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    Ok(RenderPipeline {
        pipeline,
        bind_group_layouts: layouts,
        push_mode: layout.push.clone(),
    })
}

/// Builds the compute pipeline for `shader` from its `layout`.
/// `writes_wgsl` is true on the browser's WebGPU, as `pick_source` takes it,
/// and `pad_uniforms` on a backend that pads its uniform bindings to 16 bytes.
pub(super) fn create_compute_pipeline(
    device: &wgpu::Device,
    shader: &ShaderObject,
    layout: &Layout,
    writes_wgsl: bool,
    pad_uniforms: bool,
) -> Result<ComputePipeline, RendererError> {
    let module = build_module(shader, layout, pad_uniforms, None)?;
    let PipelineSource { source, entries } = pick_source(module, writes_wgsl)?;
    let cs_entry = find_entry(&entries, naga::ShaderStage::Compute)?;
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source,
    });
    let layouts = create_bind_group_layouts(device, &layout.list_entries());
    let immediate_size = measure_immediates(layout.push.as_ref());
    let pipeline_layout =
        create_pipeline_layout(device, "Compute Pipeline Layout", &layouts, immediate_size);

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some(cs_entry),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    Ok(ComputePipeline {
        pipeline,
        bind_group_layouts: layouts,
        push_mode: layout.push.clone(),
    })
}
