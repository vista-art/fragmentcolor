//! Pads uniform bindings to 16-byte multiples for backends that require it.
//!
//! wgpu rejects a render pipeline whose uniform bindings are not 16-byte
//! aligned on backends without `BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED`, which
//! covers OpenGL and WebGL2. This pass rewrites the WGSL source so every
//! uniform type spans a multiple of 16 bytes: a struct gets a padded copy,
//! a bare uniform gets a wrapper struct and each use gains a `.v` access.
//! Binding numbers, names, and member offsets stay identical, so the
//! original module keeps driving reflection and uniform storage.

use naga::{AddressSpace, Expression, Handle, Module, Span, TypeInner};
use std::collections::HashMap;
use std::ops::Range;

/// Returns the padded source when at least one uniform binding needs it.
pub(crate) fn pad_uniform_blocks(source: &str, module: &Module) -> Option<String> {
    let ctx = module.to_ctx();
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();
    let mut padded_types: HashMap<Handle<naga::Type>, String> = HashMap::new();
    let mut wrapped: HashMap<Handle<naga::GlobalVariable>, String> = HashMap::new();

    for (handle, var) in module.global_variables.iter() {
        if var.space != AddressSpace::Uniform {
            continue;
        }
        let size = module.types[var.ty].inner.size(ctx);
        if size.is_multiple_of(16) {
            continue;
        }
        if !size.is_multiple_of(4) {
            return None;
        }
        let pad_words = ((16 - size % 16) / 4) as usize;
        let name = var.name.as_deref()?;
        let decl = range(module.global_variables.get_span(handle), source)?;
        let decl_text = &source[decl.clone()];

        if let TypeInner::Struct { .. } = module.types[var.ty].inner {
            let ty_name = module.types[var.ty].name.as_deref()?;
            let padded_name = match padded_types.get(&var.ty) {
                Some(n) => n.clone(),
                None => {
                    let ty_span = range(module.types.get_span(var.ty), source)?;
                    let text = &source[ty_span.clone()];
                    let padded_name = format!("fc_padded_{ty_name}");
                    let close = text.rfind('}')?;
                    let head = text[..close].trim_end();
                    let head = head.strip_suffix(',').unwrap_or(head);
                    let after_kw = head.find("struct")? + "struct".len();
                    let name_pos = after_kw + head[after_kw..].find(ty_name)?;
                    let mut copy = String::with_capacity(head.len() + 64);
                    copy.push_str(&head[..name_pos]);
                    copy.push_str(&padded_name);
                    copy.push_str(&head[name_pos + ty_name.len()..]);
                    copy.push(',');
                    copy.push_str(&pads(pad_words));
                    copy.push_str("\n}");
                    edits.push((ty_span.end..ty_span.end, format!("\n{copy}\n")));
                    padded_types.insert(var.ty, padded_name.clone());
                    padded_name
                }
            };
            let pos = decl_text.rfind(ty_name)?;
            let at = decl.start + pos;
            edits.push((at..at + ty_name.len(), padded_name));
        } else {
            let colon = decl_text.rfind(':')?;
            let after = &decl_text[colon + 1..];
            let type_text = after.trim().trim_end_matches(';').trim_end();
            if type_text.is_empty() {
                return None;
            }
            let wrap = format!("fc_wrap_{name}");
            let struct_decl = format!(
                "struct {wrap} {{\n    v: {type_text},{}\n}}\n",
                pads(pad_words)
            );
            let at = attributes_start(source, decl.start);
            edits.push((at..at, struct_decl));
            let type_start = decl.start + colon + 1 + (after.len() - after.trim_start().len());
            edits.push((type_start..type_start + type_text.len(), wrap));
            wrapped.insert(handle, name.to_string());
        }
    }

    if edits.is_empty() {
        return None;
    }

    let functions = module
        .functions
        .iter()
        .map(|(_, f)| f)
        .chain(module.entry_points.iter().map(|ep| &ep.function));
    for function in functions {
        for (expr_handle, expr) in function.expressions.iter() {
            let Expression::GlobalVariable(global) = expr else {
                continue;
            };
            let Some(name) = wrapped.get(global) else {
                continue;
            };
            let r = range(function.expressions.get_span(expr_handle), source)?;
            if source[r.clone()].trim() != name {
                return None;
            }
            edits.push((r.end..r.end, ".v".to_string()));
        }
    }

    edits.sort_by(|a, b| b.0.start.cmp(&a.0.start).then(b.0.end.cmp(&a.0.end)));
    let mut out = source.to_string();
    for (r, text) in edits {
        out.replace_range(r, &text);
    }
    Some(out)
}

/// Start of the `@group(...) @binding(...)` run that precedes a declaration,
/// so an inserted item lands before the attributes rather than between them.
fn attributes_start(source: &str, decl_start: usize) -> usize {
    let mut at = decl_start;
    loop {
        let head = source[..at].trim_end();
        if !head.ends_with(')') {
            return at;
        }
        let Some(open) = head.rfind('(') else {
            return at;
        };
        let before = head[..open].trim_end();
        let ident_start = before
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map(|i| i + 1)
            .unwrap_or(0);
        if ident_start == 0 || !before[..ident_start].ends_with('@') {
            return at;
        }
        at = ident_start - 1;
    }
}

fn pads(words: usize) -> String {
    (0..words)
        .map(|i| format!("\n    fc_pad{i}: u32,"))
        .collect()
}

fn range(span: Span, source: &str) -> Option<Range<usize>> {
    let r = span.to_range()?;
    (r.start < r.end && r.end <= source.len()).then_some(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    fn parse(source: &str) -> Module {
        let module = naga::front::wgsl::parse_str(source).expect("parse");
        Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&module)
            .expect("validate");
        module
    }

    fn uniform_sizes(module: &Module) -> Vec<u32> {
        let ctx = module.to_ctx();
        module
            .global_variables
            .iter()
            .filter(|(_, v)| v.space == AddressSpace::Uniform)
            .map(|(_, v)| module.types[v.ty].inner.size(ctx))
            .collect()
    }

    const TRI: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}"#;

    // Story: a bare f32 uniform becomes a 16-byte wrapper and its uses gain `.v`.
    #[test]
    fn wraps_bare_uniforms() {
        let source = format!(
            "@group(0) @binding(0) var<uniform> k: f32;\n{TRI}\n@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(k, 1.0 - k, 0.0, 1.0); }}"
        );
        let module = parse(&source);
        let padded = pad_uniform_blocks(&source, &module).expect("needs padding");
        assert!(padded.contains("struct fc_wrap_k"), "{padded}");
        assert!(padded.contains("vec4<f32>(k.v, 1.0 - k.v"), "{padded}");
        let repadded = parse(&padded);
        assert_eq!(uniform_sizes(&repadded), vec![16]);
    }

    // Story: a vec2 uniform accessed by component keeps working after the rewrite.
    #[test]
    fn wraps_vector_uniform_with_component_access() {
        let source = format!(
            "@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n{TRI}\n@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(resolution.x / resolution.y, 0.0, 0.0, 1.0); }}"
        );
        let module = parse(&source);
        let padded = pad_uniform_blocks(&source, &module).expect("needs padding");
        assert!(
            padded.contains("resolution.v.x / resolution.v.y"),
            "{padded}"
        );
        assert_eq!(uniform_sizes(&parse(&padded)), vec![16]);
    }

    // Story: a 40-byte struct gets a padded copy and the binding points at it.
    #[test]
    fn pads_struct_uniforms() {
        let source = format!(
            "struct Globals {{ resolution: vec2<f32>, mouse: vec2<f32>, drag: vec2<f32>, time: f32, zoom: f32, press: f32, pulse: f32 }}\n@group(0) @binding(0) var<uniform> u: Globals;\n{TRI}\n@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(u.press, u.pulse, u.zoom, 1.0); }}"
        );
        let module = parse(&source);
        assert_eq!(uniform_sizes(&module), vec![40]);
        let padded = pad_uniform_blocks(&source, &module).expect("needs padding");
        assert!(padded.contains("struct fc_padded_Globals"), "{padded}");
        assert!(
            padded.contains("var<uniform> u: fc_padded_Globals;"),
            "{padded}"
        );
        assert!(
            padded.contains("struct Globals {"),
            "original struct stays: {padded}"
        );
        assert_eq!(uniform_sizes(&parse(&padded)), vec![48]);
    }

    // Story: aligned uniforms need no rewrite.
    #[test]
    fn leaves_aligned_uniforms_alone() {
        let source = format!(
            "@group(0) @binding(0) var<uniform> color: vec4<f32>;\n{TRI}\n@fragment fn fs_main() -> @location(0) vec4<f32> {{ return color; }}"
        );
        let module = parse(&source);
        assert!(pad_uniform_blocks(&source, &module).is_none());
    }
}
