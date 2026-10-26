//! Immediate roots lowered to uniform buffers, the way a device without native immediates binds them.

use super::error::describe;
use super::{ShaderError, create_validator};
use naga::{AddressSpace, Module, ResourceBinding};
use std::collections::HashMap;

/// `module` with each immediate root that `bindings` names moved to a uniform at that binding of `group`.
pub(crate) fn lower_immediates(
    module: &Module,
    group: u32,
    bindings: &HashMap<String, u32>,
) -> Module {
    let mut lowered = module.clone();
    for (_, global) in lowered.global_variables.iter_mut() {
        if global.space != AddressSpace::Immediate {
            continue;
        }
        let Some(binding) = global.name.as_ref().and_then(|name| bindings.get(name)) else {
            continue;
        };
        global.space = AddressSpace::Uniform;
        global.binding = Some(ResourceBinding {
            group,
            binding: *binding,
        });
    }
    lowered
}

/// Refuses immediate roots that break the uniform layout rules.
/// The web, and any device without native immediates, binds them as uniforms,
/// so a root that only works as immediate data would draw on one device and fail on another.
pub(crate) fn check_immediates(module: &Module) -> Result<(), ShaderError> {
    let mut names: Vec<String> = module
        .global_variables
        .iter()
        .filter(|(_, global)| global.space == AddressSpace::Immediate)
        .filter_map(|(_, global)| global.name.clone())
        .collect();
    if names.is_empty() {
        return Ok(());
    }
    names.sort();
    // A group past every one the module declares, so the lowered roots take no slot of its own.
    let group = module
        .global_variables
        .iter()
        .filter_map(|(_, global)| global.binding.as_ref())
        .map(|binding| binding.group.saturating_add(1))
        .max()
        .unwrap_or(0);
    let bindings = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index as u32))
        .collect();
    let lowered = lower_immediates(module, group, &bindings);
    match create_validator().validate(&lowered) {
        Ok(_) => Ok(()),
        Err(error) => Err(ShaderError::ParseError(format!(
            "the immediate roots {} must also follow the uniform layout rules, since a device without immediates binds them as uniforms: {}",
            names.join(", "),
            describe(&error)
        ))),
    }
}
