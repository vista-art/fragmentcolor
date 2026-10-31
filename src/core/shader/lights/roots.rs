//! The light roots a render shader declares to take FragmentColor's lights.
//!
//! A shader names them `lights` and, optionally, `light_grid`, at any group
//! and binding. The renderer fills them, so they stay out of the uniform
//! storage, `list_uniforms` and `list_keys`. Two shapes of `lights` exist:
//! the read-only storage `Lights` with a runtime array, and the fixed-size
//! uniform `LightArray`, which holds an array of `N` lights.

use naga::{
    AddressSpace, ArraySize, Expression, GlobalVariable, Handle, Module, ScalarKind, ShaderStage,
    StorageAccess, Type, TypeInner, VectorSize, valid::ModuleInfo,
};
use std::num::NonZeroU32;

use crate::ShaderError;

/// The byte offset of the light array in the storage `Lights` root.
pub(crate) const LIGHTS_HEADER: u32 = 144;
/// The byte offset of the light array in the uniform `LightArray` root.
pub(crate) const ARRAY_HEADER: u32 = 32;
/// The size of one `Light`.
pub(crate) const LIGHT_SIZE: u32 = 64;

/// The layout a storage `lights` root must have, quoted in its error.
const STORAGE_LAYOUT: &str = "lights: a storage `lights` root takes `var<storage, read> lights: Lights` with \
`struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }`, \
`struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }` \
and `struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32, position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }`, \
which puts `grid` at byte 32 and the lights at byte 144, and no `@align` may raise the struct's alignment past 16";

/// The layout a `light_grid` root must have, quoted in its error.
const GRID_LAYOUT: &str =
    "light_grid: a `light_grid` root takes `var<storage, read> light_grid: array<u32>`";

/// Which `lights` root a shader declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LightShape {
    /// `var<storage, read> lights: Lights`, which holds any number of lights.
    Storage,
    /// The fixed-size `var<uniform> lights: LightArray`, which holds `count` lights.
    Array { count: u32 },
}

/// Where a light root sits.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LightBinding {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    handle: Handle<GlobalVariable>,
}

/// The light roots of one shader.
#[derive(Debug, Clone)]
pub(crate) struct LightRoots {
    pub(crate) shape: LightShape,
    pub(crate) lights: LightBinding,
    pub(crate) grid: Option<LightBinding>,
    /// The stages that read the roots: the fragment stage, and the vertex stage when it reads one.
    pub(crate) stages: wgpu::ShaderStages,
    /// Whether the module calls `arrayLength` on a root.
    measured: bool,
}

impl LightRoots {
    /// The globals the renderer fills.
    pub(crate) fn handles(&self) -> Vec<Handle<GlobalVariable>> {
        let grid = self.grid.map(|grid| grid.handle);
        std::iter::once(self.lights.handle).chain(grid).collect()
    }

    /// Whether the module measures a root with `arrayLength`, which a uniform block can't take.
    pub(crate) fn is_measured(&self) -> bool {
        self.measured
    }

    /// The most lights the root holds on its own, which only the fixed-size shape limits.
    pub(crate) fn capacity(&self) -> u32 {
        match self.shape {
            LightShape::Storage => u32::MAX,
            LightShape::Array { count } => count,
        }
    }
}

/// Finds the light roots of a module with a vertex or fragment entry point.
/// A storage `lights` or a `light_grid` with another layout is an error, and a
/// uniform `lights` that matches neither shape stays a plain uniform.
pub(crate) fn find_roots(
    module: &Module,
    info: &ModuleInfo,
) -> Result<Option<LightRoots>, ShaderError> {
    let renders = module
        .entry_points
        .iter()
        .any(|entry| matches!(entry.stage, ShaderStage::Vertex | ShaderStage::Fragment));
    if !renders {
        return Ok(None);
    }
    let Some((handle, global)) = find_global(module, "lights") else {
        return Ok(None);
    };
    let Some(binding) = global.binding.as_ref() else {
        return Ok(None);
    };
    let lights = LightBinding {
        group: binding.group,
        binding: binding.binding,
        handle,
    };
    let shape = match global.space {
        AddressSpace::Storage { access } => {
            if access != StorageAccess::LOAD || !matches_lights(module, global.ty) {
                return Err(ShaderError::TypeMismatch(STORAGE_LAYOUT.into()));
            }
            LightShape::Storage
        }
        AddressSpace::Uniform => match array_count(module, global.ty) {
            Some(count) => LightShape::Array { count },
            None => {
                log::warn!(
                    "Uniform `lights` matches neither light layout, so FragmentColor leaves it to `set`"
                );
                return Ok(None);
            }
        },
        _ => return Ok(None),
    };
    let grid = match shape {
        LightShape::Storage => find_grid(module)?,
        LightShape::Array { .. } => None,
    };
    let mut roots = LightRoots {
        shape,
        lights,
        grid,
        stages: wgpu::ShaderStages::FRAGMENT,
        measured: false,
    };
    let handles = roots.handles();
    roots.measured = measures(module, &handles);
    let vertex = module
        .entry_points
        .iter()
        .position(|entry| entry.stage == ShaderStage::Vertex);
    if let Some(index) = vertex {
        let uses = info.get_entry_point(index);
        if handles.iter().any(|handle| !uses[*handle].is_empty()) {
            roots.stages |= wgpu::ShaderStages::VERTEX;
        }
    }
    Ok(Some(roots))
}

/// Lowers the storage roots to what a device without fragment storage takes:
/// `lights` becomes a uniform with room for `count` lights, and `light_grid`
/// a private word nothing reads, since such a device never builds a grid.
pub(crate) fn lower_roots(
    module: &Module,
    roots: &LightRoots,
    count: u32,
) -> Result<Module, ShaderError> {
    if roots.shape != LightShape::Storage {
        return Ok(module.clone());
    }
    if roots.measured {
        return Err(ShaderError::TypeMismatch(
            "lights: this device takes lights through a uniform block, where `arrayLength` can't measure `lights` or `light_grid`; loop to `lights.count` instead".into(),
        ));
    }
    // The padded module is a fresh parse, so look the roots up by name.
    let Some((lights_handle, _)) = find_global(module, "lights") else {
        return Err(ShaderError::TypeMismatch(STORAGE_LAYOUT.into()));
    };
    let grid_handle = find_global(module, "light_grid").map(|(handle, _)| handle);
    let mut lowered = module.clone();
    let ty = lowered.global_variables[lights_handle].ty;
    let (name, mut members) = match &lowered.types[ty] {
        Type {
            name,
            inner: TypeInner::Struct { members, .. },
        } => (name.clone(), members.clone()),
        _ => return Err(ShaderError::TypeMismatch(STORAGE_LAYOUT.into())),
    };
    let length = NonZeroU32::new(count).unwrap_or(NonZeroU32::MIN);
    let Some(last) = members.last_mut() else {
        return Err(ShaderError::TypeMismatch(STORAGE_LAYOUT.into()));
    };
    let base = match lowered.types[last.ty].inner {
        TypeInner::Array { base, .. } => base,
        _ => return Err(ShaderError::TypeMismatch(STORAGE_LAYOUT.into())),
    };
    let span = naga::Span::UNDEFINED;
    let array = Type {
        name: None,
        inner: TypeInner::Array {
            base,
            size: ArraySize::Constant(length),
            stride: LIGHT_SIZE,
        },
    };
    last.ty = lowered.types.insert(array, span);
    let block = Type {
        name,
        inner: TypeInner::Struct {
            members,
            span: LIGHTS_HEADER + LIGHT_SIZE * length.get(),
        },
    };
    let block = lowered.types.insert(block, span);
    let lights = &mut lowered.global_variables[lights_handle];
    lights.ty = block;
    lights.space = AddressSpace::Uniform;

    if let (Some(grid), Some(_)) = (grid_handle, roots.grid) {
        let word = lowered.types.insert(
            Type {
                name: None,
                inner: TypeInner::Scalar(naga::Scalar::U32),
            },
            span,
        );
        let words = lowered.types.insert(
            Type {
                name: None,
                inner: TypeInner::Array {
                    base: word,
                    size: ArraySize::Constant(NonZeroU32::MIN),
                    stride: 4,
                },
            },
            span,
        );
        let grid = &mut lowered.global_variables[grid];
        grid.ty = words;
        grid.space = AddressSpace::Private;
        grid.binding = None;
        grid.init = None;
    }
    Ok(lowered)
}

fn find_global<'module>(
    module: &'module Module,
    name: &str,
) -> Option<(Handle<GlobalVariable>, &'module GlobalVariable)> {
    module
        .global_variables
        .iter()
        .find(|(_, global)| global.name.as_deref() == Some(name))
}

fn find_grid(module: &Module) -> Result<Option<LightBinding>, ShaderError> {
    let Some((handle, global)) = find_global(module, "light_grid") else {
        return Ok(None);
    };
    let Some(binding) = global.binding.as_ref() else {
        return Ok(None);
    };
    let words = matches!(
        module.types[global.ty].inner,
        TypeInner::Array { base, size: ArraySize::Dynamic, stride: 4 } if is_u32(module, base)
    );
    if global.space
        != (AddressSpace::Storage {
            access: StorageAccess::LOAD,
        })
        || !words
    {
        return Err(ShaderError::TypeMismatch(GRID_LAYOUT.into()));
    }
    Ok(Some(LightBinding {
        group: binding.group,
        binding: binding.binding,
        handle,
    }))
}

/// Whether any function calls `arrayLength` on one of `roots`.
fn measures(module: &Module, roots: &[Handle<GlobalVariable>]) -> bool {
    let functions = module
        .functions
        .iter()
        .map(|(_, function)| function)
        .chain(module.entry_points.iter().map(|entry| &entry.function));
    for function in functions {
        for (_, expression) in function.expressions.iter() {
            let Expression::ArrayLength(mut pointer) = *expression else {
                continue;
            };
            loop {
                match function.expressions[pointer] {
                    Expression::AccessIndex { base, .. } | Expression::Access { base, .. } => {
                        pointer = base;
                    }
                    Expression::GlobalVariable(handle) => {
                        if roots.contains(&handle) {
                            return true;
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
    }
    false
}

type Check = fn(&Module, Handle<Type>) -> bool;

const LIGHT_FIELDS: [(&str, u32, Check); 8] = [
    ("kind", 0, is_u32),
    ("intensity", 4, is_f32),
    ("range", 8, is_f32),
    ("inner_cone_cos", 12, is_f32),
    ("position", 16, is_vec3),
    ("outer_cone_cos", 28, is_f32),
    ("direction", 32, is_vec3),
    ("color", 48, is_vec3),
];

const GRID_FIELDS: [(&str, u32, Check); 7] = [
    ("view_proj", 0, is_mat4),
    ("depth_plane", 64, is_vec4),
    ("size", 80, is_uvec3),
    ("slicing", 92, is_u32),
    ("slice_scale", 96, is_f32),
    ("slice_bias", 100, is_f32),
    ("reach", 104, is_f32),
];

const LIGHTS_FIELDS: [(&str, u32, Check); 6] = [
    ("count", 0, is_u32),
    ("global_count", 4, is_u32),
    ("promoted_count", 8, is_u32),
    ("ambient", 16, is_vec3),
    ("grid", 32, is_grid),
    ("lights", LIGHTS_HEADER, is_light_list),
];

fn matches_fields(module: &Module, ty: Handle<Type>, fields: &[(&str, u32, Check)]) -> bool {
    let TypeInner::Struct { members, .. } = &module.types[ty].inner else {
        return false;
    };
    members.len() == fields.len()
        && members
            .iter()
            .zip(fields)
            .all(|(member, (name, offset, check))| {
                member.name.as_deref() == Some(*name)
                    && member.offset == *offset
                    && check(module, member.ty)
            })
}

/// Whether `ty` is the storage `Lights` struct, field by field and in size, since
/// an `@align` can raise its size past the bytes the renderer binds without moving a field.
fn matches_lights(module: &Module, ty: Handle<Type>) -> bool {
    let span = match module.types[ty].inner {
        TypeInner::Struct { span, .. } => span,
        _ => return false,
    };
    span == LIGHTS_HEADER + LIGHT_SIZE && matches_fields(module, ty, &LIGHTS_FIELDS)
}

/// The light count of the fixed-size `LightArray` shape, or `None` for any other type,
/// one whose `@align` raises its size past the bytes the renderer binds included.
fn array_count(module: &Module, ty: Handle<Type>) -> Option<u32> {
    let TypeInner::Struct { members, span } = &module.types[ty].inner else {
        return None;
    };
    let [count, ambient, lights] = members.as_slice() else {
        return None;
    };
    let header = count.name.as_deref() == Some("count")
        && count.offset == 0
        && is_u32(module, count.ty)
        && ambient.name.as_deref() == Some("ambient")
        && ambient.offset == 16
        && is_vec3(module, ambient.ty)
        && lights.name.as_deref() == Some("lights")
        && lights.offset == ARRAY_HEADER;
    match module.types[lights.ty].inner {
        TypeInner::Array {
            base,
            size: ArraySize::Constant(length),
            stride: LIGHT_SIZE,
        } if header && is_light(module, base) => {
            let length = length.get();
            let bound = LIGHT_SIZE.checked_mul(length)?.checked_add(ARRAY_HEADER)?;
            (*span == bound).then_some(length)
        }
        _ => None,
    }
}

fn is_light(module: &Module, ty: Handle<Type>) -> bool {
    matches_fields(module, ty, &LIGHT_FIELDS)
}

fn is_grid(module: &Module, ty: Handle<Type>) -> bool {
    matches_fields(module, ty, &GRID_FIELDS)
}

fn is_light_list(module: &Module, ty: Handle<Type>) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Array { base, size: ArraySize::Dynamic, stride: LIGHT_SIZE } if is_light(module, base)
    )
}

fn is_scalar(module: &Module, ty: Handle<Type>, kind: ScalarKind) -> bool {
    matches!(module.types[ty].inner, TypeInner::Scalar(scalar) if scalar.kind == kind && scalar.width == 4)
}

fn is_vector(module: &Module, ty: Handle<Type>, size: VectorSize, kind: ScalarKind) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Vector { size: found, scalar } if found == size && scalar.kind == kind && scalar.width == 4
    )
}

fn is_u32(module: &Module, ty: Handle<Type>) -> bool {
    is_scalar(module, ty, ScalarKind::Uint)
}

fn is_f32(module: &Module, ty: Handle<Type>) -> bool {
    is_scalar(module, ty, ScalarKind::Float)
}

fn is_vec3(module: &Module, ty: Handle<Type>) -> bool {
    is_vector(module, ty, VectorSize::Tri, ScalarKind::Float)
}

fn is_uvec3(module: &Module, ty: Handle<Type>) -> bool {
    is_vector(module, ty, VectorSize::Tri, ScalarKind::Uint)
}

fn is_vec4(module: &Module, ty: Handle<Type>) -> bool {
    is_vector(module, ty, VectorSize::Quad, ScalarKind::Float)
}

fn is_mat4(module: &Module, ty: Handle<Type>) -> bool {
    matches!(
        module.types[ty].inner,
        TypeInner::Matrix { columns: VectorSize::Quad, rows: VectorSize::Quad, scalar } if scalar.kind == ScalarKind::Float && scalar.width == 4
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shader;

    const LIGHT: &str = "struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32, position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }";
    const GRID: &str = "struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }";
    const LIGHTS: &str = "struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }";
    const DRAW: &str = "
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}";

    fn source(declarations: &str, body: &str) -> String {
        format!(
            "{LIGHT}\n{GRID}\n{declarations}\n{DRAW}\n@fragment fn fs_main() -> @location(0) vec4<f32> {{ {body} }}"
        )
    }

    // Story: a global on the slot of a light root fails at Shader::new, as two globals
    // on one slot do, though the renderer fills the light root itself.
    #[test]
    fn a_global_on_a_light_slot_fails() {
        let shared = source(
            &format!(
                "{LIGHTS}\n@group(0) @binding(1) var<storage, read> lights: Lights;\n@group(0) @binding(1) var<uniform> tint: vec4<f32>;"
            ),
            "return vec4<f32>(lights.ambient, 1.0);",
        );
        let err = Shader::new(shared.as_str()).unwrap_err().to_string();
        assert!(err.contains("both use @group(0) @binding(1)"), "{err}");
        assert!(Shader::new(storage_source().as_str()).is_ok());
    }

    fn array_source(count: u32) -> String {
        source(
            &format!(
                "struct LightArray {{ count: u32, ambient: vec3<f32>, lights: array<Light, {count}> }}\n@group(0) @binding(1) var<uniform> lights: LightArray;"
            ),
            "return vec4<f32>(lights.ambient + lights.lights[0].color, f32(lights.count));",
        )
    }

    fn storage_source() -> String {
        source(
            &format!(
                "{LIGHTS}\n@group(0) @binding(1) var<storage, read> lights: Lights;\n@group(0) @binding(2) var<storage, read> light_grid: array<u32>;"
            ),
            "return vec4<f32>(lights.ambient + lights.lights[light_grid[0]].color, 1.0);",
        )
    }

    fn shape(shader: &Shader) -> Option<LightShape> {
        shader.object.light_roots.as_ref().map(|roots| roots.shape)
    }

    #[test]
    fn recognizes_both_shapes() {
        for count in [32, 7] {
            let shader = Shader::new(array_source(count).as_str()).expect("array shape");
            assert_eq!(shape(&shader), Some(LightShape::Array { count }));
        }
        let shader = Shader::new(storage_source().as_str()).expect("storage shape");
        assert_eq!(shape(&shader), Some(LightShape::Storage));
        let roots = shader.object.light_roots.clone().expect("roots");
        assert_eq!((roots.lights.group, roots.lights.binding), (0, 1));
        assert_eq!(
            roots.grid.map(|grid| (grid.group, grid.binding)),
            Some((0, 2))
        );
        assert_eq!(roots.stages, wgpu::ShaderStages::FRAGMENT);
    }

    // Story: an @align that raises a light root's alignment grows its size past the bytes the
    // renderer binds, though every field keeps its offset. A storage `Lights` like that fails
    // at Shader::new, and a uniform `LightArray` like that takes no lights and stays a uniform.
    #[test]
    fn raised_light_alignment_is_no_light_root() {
        let raised = [
            LIGHTS.replace("Lights { count: u32", "Lights { @align(32) count: u32"),
            LIGHTS.replace("grid: LightGrid", "@align(32) grid: LightGrid"),
        ];
        for lights in raised {
            let declarations =
                format!("{lights}\n@group(0) @binding(1) var<storage, read> lights: Lights;");
            let body = "return vec4<f32>(lights.ambient, 1.0);";

            let result = Shader::new(source(&declarations, body).as_str());

            let Err(error) = result else {
                panic!("expected an error for {lights}");
            };
            assert!(matches!(error, ShaderError::TypeMismatch(_)), "{error:?}");
            assert!(error.to_string().contains("no `@align`"), "{error}");
        }
        let older = array_source(4).replace(
            "LightArray { count: u32",
            "LightArray { @align(64) count: u32",
        );

        let shader = Shader::new(older.as_str()).expect("a plain uniform");

        assert_eq!(shape(&shader), None);
        assert!(!shader.list_uniforms().is_empty());
    }

    #[test]
    fn light_roots_leave_the_uniform_lists() {
        let shader = Shader::new(storage_source().as_str()).expect("storage shape");
        assert!(shader.list_uniforms().is_empty());
        assert!(shader.list_keys().is_empty());
        assert!(matches!(
            shader.set("lights.count", 3u32),
            Err(ShaderError::UniformNotFound(_))
        ));
        assert!(matches!(
            shader.get::<u32>("lights.lights[0].kind"),
            Err(ShaderError::UniformNotFound(_))
        ));
        let older = Shader::new(array_source(32).as_str()).expect("array shape");
        assert!(older.list_uniforms().is_empty());
        assert!(matches!(
            older.set("lights.lights[0].color", [1.0, 0.0, 0.0]),
            Err(ShaderError::UniformNotFound(_))
        ));
    }

    #[test]
    fn ambient_round_trips() {
        for source in [storage_source(), array_source(4)] {
            let shader = Shader::new(source.as_str()).expect("lit shader");
            shader
                .set("lights.ambient", [0.1, 0.2, 0.3])
                .expect("ambient");
            let ambient: [f32; 3] = shader.get("lights.ambient").expect("ambient");
            assert_eq!(ambient, [0.1, 0.2, 0.3]);
        }
    }

    #[test]
    fn wrong_offsets_fail_or_warn() {
        // A storage `lights` with another layout fails and spells out the right one.
        let shifted = LIGHTS.replace(
            "ambient: vec3<f32>, grid",
            "ambient: vec3<f32>, extra: u32, grid",
        );
        let error = Shader::new(
            source(
                &format!("{shifted}\n@group(0) @binding(1) var<storage, read> lights: Lights;"),
                "return vec4<f32>(lights.ambient, 1.0);",
            )
            .as_str(),
        )
        .err();
        match error {
            Some(ShaderError::TypeMismatch(message)) => {
                assert!(message.contains("byte 144"), "{message}");
            }
            other => panic!("expected TypeMismatch, got {other:?}"),
        }

        // A `light_grid` of another type fails too.
        let error = Shader::new(
            source(
                &format!(
                    "{LIGHTS}\n@group(0) @binding(1) var<storage, read> lights: Lights;\n@group(0) @binding(2) var<storage, read> light_grid: array<f32>;"
                ),
                "return vec4<f32>(lights.ambient, light_grid[0]);",
            )
            .as_str(),
        )
        .err();
        assert!(
            matches!(error, Some(ShaderError::TypeMismatch(_))),
            "{error:?}"
        );

        // A uniform `lights` that matches neither shape stays a plain uniform.
        let plain = Shader::new(
            source(
                "struct Other { count: u32, ambient: vec4<f32> }\n@group(0) @binding(1) var<uniform> lights: Other;",
                "return lights.ambient;",
            )
            .as_str(),
        )
        .expect("plain uniform");
        assert_eq!(shape(&plain), None);
        assert!(plain.set("lights.count", 2u32).is_ok());
    }

    #[test]
    fn the_lowered_pbr_shader_writes_webgl2_glsl() {
        // WebGL2 compiles GLSL ES 3.0, which the lowered PBR fragment stage must become.
        let pbr = crate::Material::pbr();
        let object = &pbr.shader().object;
        let roots = object.light_roots.clone().expect("roots");
        let lowered = lower_roots(object.module_for_pipeline(true), &roots, 253).expect("lowered");
        let flags = naga::valid::ValidationFlags::all();
        let mut validator = naga::valid::Validator::new(flags, naga::valid::Capabilities::empty());
        let info = validator.validate(&lowered).expect("valid");
        let constants = naga::back::PipelineConstants::default();
        let (resolved, info) =
            naga::back::pipeline_constants::process_overrides(&lowered, &info, None, &constants)
                .expect("overrides");
        let options = naga::back::glsl::Options {
            version: naga::back::glsl::Version::Embedded {
                version: 300,
                is_webgl: true,
            },
            ..Default::default()
        };
        let pipeline = naga::back::glsl::PipelineOptions {
            shader_stage: ShaderStage::Fragment,
            entry_point: "fs_main".into(),
            multiview: None,
        };
        let mut glsl = String::new();
        naga::back::glsl::Writer::new(
            &mut glsl,
            &resolved,
            &info,
            &options,
            &pipeline,
            naga::proc::BoundsCheckPolicies::default(),
        )
        .and_then(|mut writer| writer.write())
        .expect("GLSL ES 3.0");
        assert!(glsl.contains("uniform"), "lights become a uniform block");
        assert!(
            !glsl.contains("buffer "),
            "no storage buffer remains:\n{glsl}"
        );
    }

    #[test]
    fn the_custom_page_reference_compiles() {
        // The roots and the reference functions on the Material::custom page.
        let page = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/api/scene/material/custom.md"
        ));
        let blocks: Vec<&str> = page
            .split("```wgsl")
            .skip(1)
            .filter_map(|rest| rest.split("```").next())
            .collect();
        let find = |needle: &str| {
            blocks
                .iter()
                .find(|block| block.contains(needle))
                .unwrap_or_else(|| panic!("the page should hold a block with {needle}"))
        };
        let roots = find("var<storage, read> lights");
        let functions = find("fn accumulate_lights");
        let source = format!(
            "{roots}\n{functions}
fn shade_light(lt: Light, world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {{
  return lt.color * max(dot(normal, normalize(lt.position - world)), 0.0);
}}
@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(0.0); }}
@fragment fn fs_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {{
  return vec4<f32>(accumulate_lights(p.xyz, vec3<f32>(0.0, 0.0, 1.0)), 1.0);
}}"
        );
        let shader = Shader::new(source.as_str()).expect("the reference compiles");
        assert_eq!(shape(&shader), Some(LightShape::Storage));
        let roots = shader.object.light_roots.clone().expect("roots");
        assert!(roots.grid.is_some());
    }

    #[test]
    fn compute_modules_are_untouched() {
        let shader = Shader::new(
            "struct Other { count: u32, value: f32 }
@group(0) @binding(0) var<storage, read_write> lights: Other;
@compute @workgroup_size(1) fn cs_main() { lights.value = 1.0; }",
        )
        .expect("compute shader");
        assert_eq!(shape(&shader), None);
        assert!(shader.list_uniforms().contains(&"lights".to_string()));
    }

    #[test]
    fn vertex_reads_widen_the_stages() {
        let source = format!(
            "{LIGHT}\n{GRID}\n{LIGHTS}\n@group(0) @binding(1) var<storage, read> lights: Lights;
@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(lights.ambient, 1.0); }}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(1.0); }}"
        );
        let shader = Shader::new(source.as_str()).expect("vertex reads lights");
        let roots = shader.object.light_roots.clone().expect("roots");
        assert_eq!(
            roots.stages,
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT
        );
    }

    #[test]
    fn lowering_makes_a_uniform_block() {
        let shader = Shader::new(storage_source().as_str()).expect("storage shape");
        let roots = shader.object.light_roots.clone().expect("roots");
        let lowered = lower_roots(&shader.object.module, &roots, 4).expect("lowered");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&lowered)
        .expect("the lowered module validates");
        let lights = &lowered.global_variables[roots.lights.handle];
        assert_eq!(lights.space, AddressSpace::Uniform);
        match &lowered.types[lights.ty].inner {
            TypeInner::Struct { span, .. } => assert_eq!(*span, 144 + 64 * 4),
            other => panic!("expected a struct, got {other:?}"),
        }
        let grid = roots.grid.expect("grid");
        assert_eq!(
            lowered.global_variables[grid.handle].space,
            AddressSpace::Private
        );

        let measured = Shader::new(
            source(
                &format!("{LIGHTS}\n@group(0) @binding(1) var<storage, read> lights: Lights;"),
                "return vec4<f32>(f32(arrayLength(&lights.lights)));",
            )
            .as_str(),
        )
        .expect("storage shape");
        let roots = measured.object.light_roots.clone().expect("roots");
        assert!(matches!(
            lower_roots(&measured.object.module, &roots, 4),
            Err(ShaderError::TypeMismatch(_))
        ));
    }
}
