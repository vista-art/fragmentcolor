//! Keeps `enable f16;` in the WGSL naga writes for the browser's WebGPU.

use naga::proc::TypeResolution;
use naga::valid::ModuleInfo;
use naga::{Module, Scalar, Span, Type, TypeInner};

/// Registers the `f16` type in a `module` whose `f16` values have no type,
/// such as `1.5h`. naga's WGSL writer writes `enable f16;` only for a module
/// whose types hold `f16`, and the browser refuses `f16` code without the directive.
pub(crate) fn register_half(module: &mut Module, info: &ModuleInfo) {
    let types = &module.types;
    if types
        .iter()
        .any(|(_, ty)| ty.inner.scalar() == Some(Scalar::F16))
    {
        return;
    }
    let holds_half =
        |resolution: &TypeResolution| resolution.inner_with(types).scalar() == Some(Scalar::F16);
    let functions = module
        .functions
        .iter()
        .map(|(handle, function)| (function, &info[handle]));
    let entries = module
        .entry_points
        .iter()
        .enumerate()
        .map(|(index, entry)| (&entry.function, info.get_entry_point(index)));
    // A constant or an override names its type, and a constant expression folds,
    // so only function bodies can hold an f16 value no type names.
    let mut bodies = functions.chain(entries);
    let used = bodies.any(|(function, facts)| {
        let mut handles = function.expressions.iter().map(|(handle, _)| handle);
        handles.any(|handle| holds_half(&facts[handle].ty))
    });
    if !used {
        return;
    }
    let half = Type {
        name: None,
        inner: TypeInner::Scalar(Scalar::F16),
    };
    module.types.insert(half, Span::UNDEFINED);
}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    /// The module of `source`, after `register_half`, and whether a type holds `f16`.
    fn register(source: &str) -> (Module, bool) {
        let mut module = naga::front::wgsl::parse_str(source).expect("module");
        let info = Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&module)
            .expect("info");
        register_half(&mut module, &info);
        let named = module
            .types
            .iter()
            .any(|(_, ty)| ty.inner.scalar() == Some(Scalar::F16));
        (module, named)
    }

    // Story: a module whose only f16 is an inferred literal or a conversion gets the f16 type,
    // in a function or an entry point, and a module with no f16 value gets nothing, also
    // where a constant folds `f16` away.
    #[test]
    fn half_values_register_the_type() {
        let sources = [
            "enable f16;\n@fragment fn fs_main() -> @location(0) vec4<f32> { let x = 1.5h; return vec4<f32>(f32(x)); }",
            "enable f16;\nfn halve(v: f32) -> f32 { return f32(f16(v) * 0.5h); }\n@compute @workgroup_size(1) fn cs_main() { _ = halve(1.0); }",
            "enable f16;\n@group(0) @binding(0) var<storage, read_write> out: array<f32, 2>;\n@compute @workgroup_size(1) fn cs_main() { out[0] = f32(f16(out[1])); }",
        ];
        for source in sources {
            assert!(register(source).1, "{source}");
        }

        let plain = [
            "@compute @workgroup_size(1) fn cs_main() { let x = 1.5; _ = x; }",
            "enable f16;\nconst scale = f32(2.5h);\n@compute @workgroup_size(1) fn cs_main() { _ = scale; }",
        ];
        for source in plain {
            assert!(!register(source).1, "{source}");
        }
    }
}
