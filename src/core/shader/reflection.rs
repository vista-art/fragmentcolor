//! Parses and validates WGSL, and reflects a module's globals
//! into the uniforms, textures and buffers a shader holds.

use super::error::Source;
use super::{Diagnostic, PushEntry, ShaderError, ShaderHash, StorageEntry};
use super::{Uniform, UniformData, convert_type};
use crate::core::renderer::device::CANDIDATES;
use naga::front::wgsl::{Frontend, Options};
use naga::{
    AddressSpace, Module,
    valid::{Capabilities, ModuleInfo, ValidationFlags, Validator},
};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use wgpu::DownlevelFlags;
use wgpu_naga_bridge::features_to_naga_capabilities;

/// The naga capabilities of the features a device may open with, on any backend.
fn read_capabilities() -> Capabilities {
    features_to_naga_capabilities(CANDIDATES, DownlevelFlags::all())
}

/// A WGSL parser that takes only what a device may support.
fn create_parser() -> Frontend {
    let mut options = Options::new();
    options.capabilities = read_capabilities();
    Frontend::new_with_options(options)
}

/// A validator that refuses what no device FragmentColor opens can run,
/// such as `f16`, subgroups, `texture_external` or texture atomics.
pub(crate) fn create_validator() -> Validator {
    Validator::new(ValidationFlags::all(), read_capabilities())
}

/// Parses WGSL `source`, or returns naga's report on a source it refuses.
pub(crate) fn parse_wgsl(source: &str) -> Result<Module, ShaderError> {
    match create_parser().parse(source) {
        Ok(module) => Ok(module),
        Err(error) => Err(ShaderError::WgslParseError(Diagnostic::from_parse(
            &error, source,
        ))),
    }
}

/// Validates `module`, or returns naga's report on one it refuses,
/// with its places located in `source` when the module came from it.
pub(crate) fn validate_module(
    module: &Module,
    source: Option<Source<'_>>,
) -> Result<ModuleInfo, ShaderError> {
    match create_validator().validate(module) {
        Ok(info) => Ok(info),
        Err(error) => Err(ShaderError::ValidationError(Diagnostic::from_validation(
            &error, source,
        ))),
    }
}

pub(super) fn hash(source: &str) -> ShaderHash {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let slice = hasher.finalize();

    slice.into()
}

/// Collect the set of image-class global variables that are ever consumed by an
/// `ImageSample` expression. Textures outside this set are only read via
/// `textureLoad`/image ops, and the bind-group layout can request a non-filterable
/// sample type (unlocking Rgba32Float and similar as sampled sources without
/// the `FLOAT32_FILTERABLE` feature).
///
/// If any `ImageSample` argument cannot be resolved to a direct global (e.g. it
/// comes from a function parameter), every image global is conservatively
/// reported as sampled — filterable=true is always a safe layout, filterable=false
/// is not.
fn sampled_image_globals(module: &Module) -> HashSet<naga::Handle<naga::GlobalVariable>> {
    use naga::Expression;

    let mut sampled: HashSet<naga::Handle<naga::GlobalVariable>> = Default::default();
    let mut indirect = false;

    let mut visit = |function: &naga::Function| {
        for (_, expr) in function.expressions.iter() {
            if let Expression::ImageSample { image, .. } = expr {
                match &function.expressions[*image] {
                    Expression::GlobalVariable(handle) => {
                        sampled.insert(*handle);
                    }
                    _ => indirect = true,
                }
            }
        }
    };

    for (_, f) in module.functions.iter() {
        visit(f);
    }
    for ep in &module.entry_points {
        visit(&ep.function);
    }

    if indirect {
        for (handle, var) in module.global_variables.iter() {
            if matches!(&module.types[var.ty].inner, naga::TypeInner::Image { .. }) {
                sampled.insert(handle);
            }
        }
    }

    sampled
}

/// Records that `name` holds the slot `binding` names, refusing a slot another global holds.
/// naga only refuses two globals on one slot when one entry point uses both.
fn claim_slot(
    slots: &mut HashMap<(u32, u32), String>,
    binding: &naga::ResourceBinding,
    name: &str,
) -> Result<(), ShaderError> {
    let slot = (binding.group, binding.binding);
    match slots.insert(slot, name.to_string()) {
        Some(first) => Err(ShaderError::ParseError(format!(
            "`{first}` and `{name}` both use @group({}) @binding({})",
            binding.group, binding.binding
        ))),
        None => Ok(()),
    }
}

/// The stages whose entry points use the global `handle`.
fn find_stages(
    module: &Module,
    info: &ModuleInfo,
    handle: naga::Handle<naga::GlobalVariable>,
) -> wgpu::ShaderStages {
    let mut stages = wgpu::ShaderStages::NONE;
    for (index, entry) in module.entry_points.iter().enumerate() {
        if info.get_entry_point(index)[handle].is_empty() {
            continue;
        }
        stages |= match entry.stage {
            naga::ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            naga::ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
            naga::ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
            _ => wgpu::ShaderStages::NONE,
        };
    }
    stages
}

/// Refuses a filtering sampler that samples a depth texture. WebGPU binds a depth texture
/// only with a comparison sampler or one that doesn't filter, and a FragmentColor
/// `sampler` filters, so such a pair would draw on native backends and fail on the web.
pub(super) fn check_samplers(module: &Module, info: &ModuleInfo) -> Result<(), ShaderError> {
    for index in 0..module.entry_points.len() {
        for key in info.get_entry_point(index).sampling_set.iter() {
            let image = &module.global_variables[key.image];
            let sampler = &module.global_variables[key.sampler];
            let depth = matches!(
                module.types[image.ty].inner,
                naga::TypeInner::Image {
                    class: naga::ImageClass::Depth { .. },
                    ..
                }
            );
            let filtering = matches!(
                module.types[sampler.ty].inner,
                naga::TypeInner::Sampler { comparison: false }
            );
            if depth && filtering {
                return Err(ShaderError::ParseError(format!(
                    "`{}` filters, and it samples the depth texture `{}`, which only a sampler_comparison or textureLoad reads on the web",
                    sampler.name.as_deref().unwrap_or("?"),
                    image.name.as_deref().unwrap_or("?")
                )));
            }
        }
    }
    Ok(())
}

/// Whether the global writes to its storage, as a `read_write` buffer or a writable storage texture.
fn writes_storage(module: &Module, variable: &naga::GlobalVariable) -> bool {
    match variable.space {
        AddressSpace::Storage { access } => access.contains(naga::StorageAccess::STORE),
        _ => matches!(
            module.types[variable.ty].inner,
            naga::TypeInner::Image {
                class: naga::ImageClass::Storage { access, .. },
                ..
            } if access.contains(naga::StorageAccess::STORE)
        ),
    }
}

/// Reflects the module's bindings, leaving out the `filled` globals, which the renderer fills.
pub(super) fn parse_uniforms(
    module: &Module,
    info: &ModuleInfo,
    filled: &[naga::Handle<naga::GlobalVariable>],
) -> Result<HashMap<String, Uniform>, ShaderError> {
    let mut uniforms = HashMap::new();
    let mut slots: HashMap<(u32, u32), String> = HashMap::new();
    let sampled_set = sampled_image_globals(module);

    for (handle, variable) in module.global_variables.iter() {
        if filled.contains(&handle) {
            // The renderer fills a light root, which still holds its slot.
            if let (Some(name), Some(binding)) = (&variable.name, &variable.binding) {
                claim_slot(&mut slots, binding, name)?;
            }
            continue;
        }
        let stages = find_stages(module, info, handle);
        // Handle WorkGroup specially: ignore unbound; error on bound (unexpected)
        match variable.space {
            AddressSpace::WorkGroup => {
                if variable.binding.is_some() {
                    return Err(ShaderError::ParseError(
                        "Bound workgroup variables are not supported".into(),
                    ));
                } else {
                    continue;
                }
            }
            AddressSpace::Immediate => {
                // Parse immediate data (formerly WGSL push constants) as a dedicated variant;
                // no binding is expected.
                let uniform_name = variable
                    .name
                    .clone()
                    .ok_or(ShaderError::ParseError("Unnamed immediate variable".into()))?;
                let ty = &module.types[variable.ty];
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                uniforms.insert(
                    uniform_name.clone(),
                    Uniform {
                        name: uniform_name,
                        group: 0,
                        binding: 0,
                        stages,
                        data: UniformData::PushConstant(vec![PushEntry {
                            inner: vec![inner],
                            span,
                        }]),
                    },
                );
                continue;
            }
            _ => {}
        }

        // Accept classic uniform buffers and handle-class resources (textures/samplers/storage)
        match variable.space {
            AddressSpace::Uniform | AddressSpace::Handle | AddressSpace::Storage { .. } => {}
            _ => {
                continue;
            }
        }

        let uniform_name = variable
            .name
            .clone()
            .ok_or(ShaderError::ParseError("Unnamed uniform".into()))?;

        let binding = variable
            .binding
            .as_ref()
            .ok_or(ShaderError::ParseError("Missing binding".into()))?;
        claim_slot(&mut slots, binding, &uniform_name)?;
        // No device FragmentColor opens takes writable storage in the vertex stage.
        if stages.contains(wgpu::ShaderStages::VERTEX) && writes_storage(module, variable) {
            return Err(ShaderError::ParseError(format!(
                "`{uniform_name}` is writable storage, which a @vertex entry point can't use"
            )));
        }

        let ty = &module.types[variable.ty];

        // Special handling for Storage: wrap inner shape and carry access flags
        let data = match variable.space {
            AddressSpace::Storage { access } => {
                // convert_type will yield a Struct/Array/etc. shape; wrap it
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                UniformData::Storage(vec![StorageEntry {
                    inner: vec![inner],
                    span,
                    access: access.into(),
                }])
            }
            _ => {
                let mut d = convert_type(module, ty)?;
                if let UniformData::Texture(ref mut meta) = d {
                    meta.sampled = sampled_set.contains(&handle);
                }
                d
            }
        };

        uniforms.insert(
            uniform_name.clone(),
            Uniform {
                name: uniform_name,
                group: binding.group,
                binding: binding.binding,
                stages,
                data,
            },
        );
    }

    Ok(uniforms)
}
