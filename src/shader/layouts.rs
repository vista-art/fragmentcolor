//! Explicit struct layouts kept in the WGSL the browser's WebGPU gets.
//!
//! There wgpu hands the browser the WGSL that naga writes back from the module,
//! and that writer drops `@size` and `@align`. A buffer struct whose members sit
//! past where their types alone put them would then change layout on the web.
//! `write_layouts` writes the module as wgpu does and puts back a `@size` on each
//! member of such a struct, which pins every member and the struct's size.

use super::ShaderError;
use super::error::describe;
use naga::keywords::wgsl::{BUILTIN_IDENTIFIER_SET, RESERVED_SET};
use naga::proc::{CaseInsensitiveKeywordSet, Layouter, NameKey, Namer};
use naga::valid::ModuleInfo;
use naga::{AddressSpace, ArraySize, FastHashMap, Handle, Module, ShaderStage, Type, TypeInner};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Why the WGSL the browser's WebGPU gets can't keep a module's explicit layouts.
#[derive(Debug, Error)]
pub(crate) enum LayoutLoss {
    /// A pinned struct that the text leaves out or lays out otherwise, as `reason` says.
    #[error("struct `{name}` loses its layout in the WGSL the web gets: {reason}")]
    Moved { name: String, reason: String },
    #[error(transparent)]
    Layout(#[from] naga::proc::LayoutError),
    #[error(transparent)]
    Write(#[from] naga::back::wgsl::Error),
    #[error(transparent)]
    Parse(#[from] naga::front::wgsl::ParseError),
}

impl From<LayoutLoss> for ShaderError {
    fn from(loss: LayoutLoss) -> Self {
        match loss {
            LayoutLoss::Write(error) => error.into(),
            LayoutLoss::Parse(error) => error.into(),
            other => ShaderError::LayoutError(describe(&other)),
        }
    }
}

/// The WGSL the browser gets for `module`: what wgpu writes from it, with a `@size` on each
/// member of a buffer struct whose explicit layout that text would lose. `None` when no
/// struct needs one, and the module goes as it is. `info` is the module's validation.
///
/// It reads the text back and refuses a struct whose layout it doesn't keep,
/// naming the struct and the first member that moves.
pub(crate) fn write_layouts(
    module: &Module,
    info: &ModuleInfo,
) -> Result<Option<String>, LayoutLoss> {
    let layouter = measure_types(module)?;
    let pinned = find_explicit(module, &layouter);
    if pinned.is_empty() {
        return Ok(None);
    }
    let names = name_pinned(module, &pinned);
    let flags = naga::back::wgsl::WriterFlags::empty();
    let written = naga::back::wgsl::write_string(module, info, flags)?;
    let text = pin_sizes(module, &names, &written)?;
    check_layouts(module, &names, &text)?;
    Ok(Some(text))
}

/// The names of the buffer structs whose layout `@size` or `@align` sets,
/// which a backend that drops those attributes can't keep.
pub(crate) fn list_explicit(module: &Module) -> Result<Vec<String>, LayoutLoss> {
    let layouter = measure_types(module)?;
    let pinned = find_explicit(module, &layouter);
    Ok(pinned
        .into_iter()
        .map(|handle| name_struct(module, handle))
        .collect())
}

/// The name naga's WGSL writer gives each entry point of `module`, with its stage.
///
/// wgpu hands the browser's WebGPU that writer's text, so a pipeline there asks for these
/// names. They differ from the module's where the writer renames one: `blur2` is `blur2_`,
/// `blur_` is `blur`, and `cs_main` next to a struct named `cs_main_` is `cs_main_1`.
pub(crate) fn name_entries(module: &Module) -> Vec<(ShaderStage, String)> {
    let names = name_module(module);
    module
        .entry_points
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let key = u16::try_from(index).ok().map(NameKey::EntryPoint);
            let name = key.and_then(|key| names.get(&key));
            let name = name.cloned().unwrap_or_else(|| entry.name.clone());
            (entry.stage, name)
        })
        .collect()
}

/// The size and alignment of each type of `module`, by its member types.
fn measure_types(module: &Module) -> Result<Layouter, LayoutLoss> {
    let mut layouter = Layouter::default();
    layouter.update(module.to_ctx())?;
    Ok(layouter)
}

/// The buffer structs whose members or size sit past where their member types put them.
fn find_explicit(module: &Module, layouter: &Layouter) -> Vec<Handle<Type>> {
    let mut found: Vec<Handle<Type>> = find_buffers(module)
        .into_iter()
        .filter(|handle| is_explicit(module, layouter, *handle))
        .collect();
    found.sort_by_key(|handle| handle.index());
    found
}

/// Every name naga's WGSL writer gives in `module`, from a run
/// of the namer that writer uses, set up as the writer sets it up.
fn name_module(module: &Module) -> FastHashMap<NameKey, String> {
    let mut names = FastHashMap::default();
    Namer::default().reset(
        module,
        &RESERVED_SET,
        &BUILTIN_IDENTIFIER_SET,
        CaseInsensitiveKeywordSet::empty(),
        &["__", "_naga"],
        &mut names,
    );
    names
}

/// The names naga's WGSL writer gives the `pinned` structs.
///
/// That writer renames a name that repeats another, holds `__`, isn't ASCII,
/// or ends in a digit or an underscore. `check_layouts` reads the text back under
/// these names, so a namer that names otherwise refuses the module instead of pinning the wrong struct.
fn name_pinned(module: &Module, pinned: &[Handle<Type>]) -> HashMap<Handle<Type>, String> {
    let names = name_module(module);
    pinned
        .iter()
        .map(|handle| {
            let name = names.get(&NameKey::Type(*handle)).cloned();
            (
                *handle,
                name.unwrap_or_else(|| name_struct(module, *handle)),
            )
        })
        .collect()
}

/// The structs a uniform, storage or immediate root holds, at any depth.
fn find_buffers(module: &Module) -> HashSet<Handle<Type>> {
    let mut found = HashSet::new();
    let mut stack: Vec<Handle<Type>> = module
        .global_variables
        .iter()
        .filter(|(_, global)| {
            matches!(
                global.space,
                AddressSpace::Uniform | AddressSpace::Storage { .. } | AddressSpace::Immediate
            )
        })
        .map(|(_, global)| global.ty)
        .collect();
    while let Some(ty) = stack.pop() {
        match &module.types[ty].inner {
            TypeInner::Struct { members, .. } => {
                if found.insert(ty) {
                    stack.extend(members.iter().map(|member| member.ty));
                }
            }
            TypeInner::Array { base, .. } => stack.push(*base),
            _ => {}
        }
    }
    found
}

/// Whether a member of the struct `handle`, or its size, sits past where its member types put it.
fn is_explicit(module: &Module, layouter: &Layouter, handle: Handle<Type>) -> bool {
    let TypeInner::Struct { members, span } = &module.types[handle].inner else {
        return false;
    };
    let mut cursor = 0;
    for member in members {
        let layout = layouter[member.ty];
        if layout.alignment.round_up(cursor) != member.offset {
            return true;
        }
        cursor = member.offset + layout.size;
    }
    layouter[handle].alignment.round_up(cursor) != *span
}

/// `written` with a `@size` on each member of the pinned structs, which `names` lists
/// under the names they take in it: the bytes up to the next member, or to the end
/// of the struct for the last one. A runtime-sized array keeps its open size.
fn pin_sizes(
    module: &Module,
    names: &HashMap<Handle<Type>, String>,
    written: &str,
) -> Result<String, LayoutLoss> {
    let mut lines: Vec<String> = written.lines().map(str::to_string).collect();
    for (handle, written_name) in names {
        let TypeInner::Struct { members, span } = &module.types[*handle].inner else {
            continue;
        };
        let header = format!("struct {written_name} {{");
        let missing = || LayoutLoss::Moved {
            name: name_struct(module, *handle),
            reason: format!("naga's writer leaves out `struct {written_name}`"),
        };
        let start = lines
            .iter()
            .position(|line| line.trim_end() == header)
            .ok_or_else(missing)?;
        let body = lines
            .get_mut(start + 1..start + 1 + members.len())
            .ok_or_else(missing)?;
        let ends = members
            .iter()
            .skip(1)
            .map(|member| member.offset)
            .chain([*span]);
        for ((line, member), end) in body.iter_mut().zip(members).zip(ends) {
            if is_open(module, member.ty) {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            line.insert_str(indent, &format!("@size({}) ", end - member.offset));
        }
    }
    let mut text = lines.join("\n");
    text.push('\n');
    Ok(text)
}

/// Whether `ty` is a runtime-sized array, which takes no `@size`.
fn is_open(module: &Module, ty: Handle<Type>) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Array {
            size: ArraySize::Dynamic,
            ..
        }
    )
}

/// Refuses `text` when a pinned struct read back from it, under the name `names` gives it,
/// has another size or puts a member elsewhere.
fn check_layouts(
    module: &Module,
    names: &HashMap<Handle<Type>, String>,
    text: &str,
) -> Result<(), LayoutLoss> {
    let read = naga::front::wgsl::parse_str(text)?;
    for (handle, written_name) in names {
        let name = name_struct(module, *handle);
        let TypeInner::Struct { members, span } = &module.types[*handle].inner else {
            continue;
        };
        let found = read.types.iter().find_map(|(_, ty)| match &ty.inner {
            TypeInner::Struct { members, span }
                if ty.name.as_deref() == Some(written_name.as_str()) =>
            {
                Some((members, *span))
            }
            _ => None,
        });
        let Some((web, web_span)) = found else {
            let reason = format!("naga's parser finds no `struct {written_name}`");
            return Err(LayoutLoss::Moved { name, reason });
        };
        for (member, moved) in members.iter().zip(web) {
            if member.offset != moved.offset {
                let member_name = member.name.as_deref().unwrap_or("?");
                let reason = format!(
                    "there `{member_name}` sits at byte {}, not {}",
                    moved.offset, member.offset
                );
                return Err(LayoutLoss::Moved { name, reason });
            }
        }
        if *span != web_span {
            let reason = format!("there the struct spans {web_span} bytes, not {span}");
            return Err(LayoutLoss::Moved { name, reason });
        }
    }
    Ok(())
}

/// How errors name the struct `handle`: by the name the shader gives it.
fn name_struct(module: &Module, handle: Handle<Type>) -> String {
    module.types[handle]
        .name
        .clone()
        .unwrap_or_else(|| format!("{handle:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shader;

    /// Each struct's span and the offset of each of its members.
    /// Names are left out, since naga's writer renames a struct whose name it can't keep.
    type Layouts = Vec<(u32, Vec<u32>)>;

    /// The layouts of `module`, sorted.
    fn read_layouts(module: &Module) -> Layouts {
        let mut layouts: Layouts = module
            .types
            .iter()
            .filter_map(|(_, ty)| match &ty.inner {
                TypeInner::Struct { members, span } => {
                    Some((*span, members.iter().map(|member| member.offset).collect()))
                }
                _ => None,
            })
            .collect();
        layouts.sort();
        layouts
    }

    /// The module of the shader `source` and its validation.
    fn compile(source: &str) -> (Module, ModuleInfo) {
        let shader = Shader::new(source).expect("shader");
        let module = shader.object.module.clone();
        let info = super::super::create_validator()
            .validate(&module)
            .expect("validates");
        (module, info)
    }

    /// The plain WGSL wgpu writes for the browser from `module`.
    fn write_plain(module: &Module, info: &ModuleInfo) -> String {
        let flags = naga::back::wgsl::WriterFlags::empty();
        naga::back::wgsl::write_string(module, info, flags).expect("writes WGSL")
    }

    /// Asserts that each entry point of `module` sits in `text` under the name `name_entries` gives it.
    fn assert_entries(module: &Module, text: &str) {
        for (_, name) in name_entries(module) {
            let header = format!("fn {name}(");
            assert!(text.contains(&header), "no `{header}` in\n{text}");
        }
    }

    const SOURCES: [&str; 9] = [
        "struct Tail { a: vec4<f32>, @size(32) b: f32 };
@group(0) @binding(0) var<uniform> tail: Tail;
@compute @workgroup_size(1) fn cs_main() { let x = tail.b; }",
        "struct Gaps { a: f32, @align(16) b: vec2<f32>, c: u32, @align(32) d: vec4<f32> };
@group(0) @binding(0) var<storage, read_write> gaps: Gaps;
@compute @workgroup_size(1) fn cs_main() { gaps.c = 1u; gaps.d = vec4<f32>(gaps.a); }",
        "struct Inner { @align(16) a: f32, @size(12) b: f32 };
struct Outer { x: f32, inner: Inner, y: f32 };
@group(0) @binding(0) var<storage, read_write> outer: Outer;
@compute @workgroup_size(1) fn cs_main() { outer.inner = Inner(outer.x, outer.y); }",
        "struct Item { @size(20) v: vec2<f32>, w: f32 };
struct Items { items: array<Item, 3>, count: u32 };
@group(0) @binding(0) var<storage, read_write> list: Items;
@compute @workgroup_size(1) fn cs_main() { list.items[2].w = f32(list.count); }",
        // Names naga's writer changes: a trailing digit, a trailing underscore, non-ASCII letters.
        "struct Params2 { a: f32, @align(16) b: vec2<f32> };
struct Light_ { @size(32) c: f32, d: f32 };
struct Données { @align(16) e: f32 };
@group(0) @binding(0) var<uniform> params: Params2;
@group(0) @binding(1) var<uniform> light: Light_;
@group(0) @binding(2) var<uniform> données: Données;
@compute @workgroup_size(1) fn cs_main() { let x = params.b.x + light.d + données.e; }",
        // A struct the writer shortens, `Tile_` to `Tile`, a pinned struct that repeats
        // a name, `Params_` as `Params_1` after `Params`, and an entry point `blur2` as `blur2_`.
        "struct Tile_ { a: vec4<f32> };
struct Params { a: f32, @align(16) b: vec2<f32> };
struct Params_ { @size(32) c: f32, d: f32 };
@group(0) @binding(0) var<uniform> user: Tile_;
@group(0) @binding(1) var<uniform> pinned: Params;
@group(0) @binding(2) var<uniform> later: Params_;
@compute @workgroup_size(1) fn blur2() { let x = user.a.x + pinned.b.x + later.d; }",
        // An entry point the writer shortens, `blur_` to `blur`.
        "struct Params { a: f32, @align(16) b: vec2<f32> };
@group(0) @binding(0) var<uniform> pinned: Params;
@compute @workgroup_size(1) fn blur_() { let x = pinned.b.x; }",
        // An entry point a buffer struct takes the name of: `cs_main_` writes as `cs_main`,
        // so the entry point `cs_main` writes as `cs_main_1`.
        "struct cs_main_ { a: vec4<f32> };
struct Params { a: f32, @align(16) b: vec2<f32> };
@group(0) @binding(0) var<uniform> user: cs_main_;
@group(0) @binding(1) var<uniform> pinned: Params;
@compute @workgroup_size(1) fn cs_main() { let x = user.a.x + pinned.b.x; }",
        // Two names that collapse to one: the writer names them `Foo` and `Foo_1`.
        "struct Foo_ { a: vec4<f32> };
struct Foo { a: f32, @align(16) b: f32 };
@group(0) @binding(0) var<uniform> plain: Foo_;
@group(0) @binding(1) var<uniform> pinned: Foo;
@compute @workgroup_size(1) fn cs_main() { let x = plain.a.x + pinned.b; }",
    ];

    // Story: the plain WGSL wgpu writes for the browser drops @size and @align,
    // so each of these structs would take another layout on the web.
    #[test]
    fn plain_wgsl_loses_explicit_layouts() {
        for source in SOURCES {
            let (module, info) = compile(source);
            let web = naga::front::wgsl::parse_str(&write_plain(&module, &info)).expect("parses");
            assert_ne!(read_layouts(&module), read_layouts(&web), "{source}");
        }
    }

    // Story: @size on the last member, @align gaps, nested structs, arrays of structs and
    // names the writer would change keep every member's offset and every struct's size in
    // the WGSL the web gets, and each entry point sits there under the name the pipeline asks for.
    #[test]
    fn explicit_layouts_survive_the_web() {
        for source in SOURCES {
            let (module, info) = compile(source);
            let text = write_layouts(&module, &info)
                .expect("writes")
                .expect("pins sizes");
            let web = naga::front::wgsl::parse_str(&text).expect("parses");
            assert_eq!(
                read_layouts(&module),
                read_layouts(&web),
                "{source}\n{text}"
            );
            let checked = super::super::create_validator().validate(&web);
            assert!(checked.is_ok(), "{checked:?}\n{text}");
            assert_entries(&module, &text);
        }
    }

    // Story: the writer renames an entry point that ends in a digit or an underscore, or whose
    // name a type took, in the WGSL wgpu writes from any module, so the browser's pipelines ask
    // for the names `name_entries` gives, which match the text.
    #[test]
    fn entry_points_take_the_names_the_writer_gives() {
        let cases = [
            ("@compute @workgroup_size(1) fn blur2() {}", "blur2_"),
            ("@compute @workgroup_size(1) fn blur_() {}", "blur"),
            (
                "struct cs_main_ { a: vec4<f32> };
@group(0) @binding(0) var<uniform> user: cs_main_;
@compute @workgroup_size(1) fn cs_main() { let x = user.a.x; }",
                "cs_main_1",
            ),
            ("@compute @workgroup_size(1) fn cs_main() {}", "cs_main"),
        ];
        for (source, expected) in cases {
            let (module, info) = compile(source);
            let names = name_entries(&module);
            assert_eq!(names, vec![(ShaderStage::Compute, expected.to_string())]);
            assert_entries(&module, &write_plain(&module, &info));
        }
    }

    // Story: a struct that ends in a runtime-sized array keeps its open size and its layout.
    #[test]
    fn runtime_arrays_keep_their_open_size() {
        let source = "struct Buf { @size(16) count: u32, data: array<u32> };
@group(0) @binding(0) var<storage, read_write> buf: Buf;
@compute @workgroup_size(1) fn cs_main() { buf.data[0] = arrayLength(&buf.data); }";
        let module = naga::front::wgsl::parse_str(source).expect("parses");
        let info = super::super::create_validator()
            .validate(&module)
            .expect("validates");
        let text = write_layouts(&module, &info)
            .expect("writes")
            .expect("pins sizes");
        let web = naga::front::wgsl::parse_str(&text).expect("parses");
        assert_eq!(read_layouts(&module), read_layouts(&web), "{text}");
        let checked = super::super::create_validator().validate(&web);
        assert!(checked.is_ok(), "{checked:?}\n{text}");
    }

    // Story: WGSL whose struct takes another layout is refused, naming the struct and the member.
    #[test]
    fn a_layout_the_web_loses_fails() {
        let (module, info) = compile(SOURCES[1]);
        let layouter = measure_types(&module).expect("measures");
        let pinned = find_explicit(&module, &layouter);
        let names: HashMap<Handle<Type>, String> = pinned
            .iter()
            .map(|handle| (*handle, name_struct(&module, *handle)))
            .collect();

        let loss = check_layouts(&module, &names, &write_plain(&module, &info)).unwrap_err();

        let LayoutLoss::Moved { name, reason } = loss else {
            panic!("expected a moved struct, got {loss:?}");
        };
        assert_eq!(name, "Gaps");
        assert_eq!(reason, "there `b` sits at byte 8, not 16");
    }

    // Story: a struct that sits where its types put it, or one no buffer holds, keeps the plain WGSL.
    #[test]
    fn natural_layouts_keep_plain_wgsl() {
        let (module, info) = compile(
            "struct Plain { a: f32, b: vec4<f32> };
struct Local { a: f32, @align(16) b: f32 };
@group(0) @binding(0) var<uniform> plain: Plain;
@compute @workgroup_size(1) fn cs_main() { var local = Local(1.0, 2.0); let x = plain.b + local.b; }",
        );
        assert!(write_layouts(&module, &info).expect("writes").is_none());
        assert!(list_explicit(&module).expect("lists").is_empty());
    }
}
