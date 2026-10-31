//! The `@location` inputs a shader's vertex entry point reads,
//! and the WGSL `Shader::from_vertex` builds from one vertex.

use super::{ShaderError, ShaderObject};
use crate::{Vertex, VertexData};

/// Descriptor for a shader vertex input parameter reflected from WGSL/Naga.
#[derive(Debug, Clone)]
pub(crate) struct VertexInputDesc {
    pub name: String,
    pub location: u32,
    pub format: wgpu::VertexFormat,
}

impl ShaderObject {
    /// The `@location` inputs of the vertex entry point, by location. Builtins are skipped.
    /// A struct argument gives its `@location` members, each under its own name.
    pub(crate) fn reflect_vertex_inputs(&self) -> Result<Vec<VertexInputDesc>, ShaderError> {
        let mut inputs: Vec<VertexInputDesc> = Vec::new();

        // The same entry point create_render_pipeline runs.
        if let Some(entry_point) = self.find_entry_point(naga::ShaderStage::Vertex) {
            for argument in entry_point.function.arguments.iter() {
                match argument.binding.as_ref() {
                    Some(binding) => {
                        self.push_input(&mut inputs, binding, argument.ty, &argument.name)?
                    }
                    None => self.push_members(&mut inputs, argument.ty)?,
                }
            }
        }
        // Sort by location to keep a stable order
        inputs.sort_by_key(|vertex_input| vertex_input.location);
        Ok(inputs)
    }

    /// Adds the `@location` members of the struct `ty` to `inputs`.
    fn push_members(
        &self,
        inputs: &mut Vec<VertexInputDesc>,
        ty: naga::Handle<naga::Type>,
    ) -> Result<(), ShaderError> {
        let naga::TypeInner::Struct { members, .. } = &self.module.types[ty].inner else {
            return Ok(());
        };
        for member in members {
            if let Some(binding) = member.binding.as_ref() {
                self.push_input(inputs, binding, member.ty, &member.name)?;
            }
        }
        Ok(())
    }

    /// Adds one input to `inputs` when `binding` is a `@location`, named `attr<N>` when it has no name.
    fn push_input(
        &self,
        inputs: &mut Vec<VertexInputDesc>,
        binding: &naga::Binding,
        ty: naga::Handle<naga::Type>,
        name: &Option<String>,
    ) -> Result<(), ShaderError> {
        let naga::Binding::Location { location, .. } = binding else {
            return Ok(());
        };
        let format = naga_ty_to_vertex_format(&self.module.types[ty])?;
        let name = name.clone().unwrap_or_else(|| format!("attr{location}"));
        // Each input reads the mesh attribute of its name, so two inputs can't share one.
        if let Some(first) = inputs.iter().find(|input| input.name == name) {
            return Err(ShaderError::ParseError(format!(
                "the vertex inputs at @location({}) and @location({location}) are both named `{name}`, and each input reads the mesh attribute of its name",
                first.location
            )));
        }
        inputs.push(VertexInputDesc {
            name,
            location: *location,
            format,
        });
        Ok(())
    }
}

fn naga_ty_to_vertex_format(ty: &naga::Type) -> Result<wgpu::VertexFormat, ShaderError> {
    use naga::{ScalarKind, TypeInner, VectorSize};
    match &ty.inner {
        TypeInner::Scalar(s) => match s.kind {
            ScalarKind::Float if s.width == 4 => Ok(wgpu::VertexFormat::Float32),
            ScalarKind::Sint if s.width == 4 => Ok(wgpu::VertexFormat::Sint32),
            ScalarKind::Uint if s.width == 4 => Ok(wgpu::VertexFormat::Uint32),
            _ => Err(ShaderError::TypeMismatch(
                "Unsupported scalar width/kind for vertex input".into(),
            )),
        },
        TypeInner::Vector { size, scalar, .. } => match (size, scalar.kind, scalar.width) {
            (VectorSize::Bi, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x2),
            (VectorSize::Tri, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x3),
            (VectorSize::Quad, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x4),
            (VectorSize::Bi, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x2),
            (VectorSize::Tri, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x3),
            (VectorSize::Quad, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x4),
            (VectorSize::Bi, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x2),
            (VectorSize::Tri, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x3),
            (VectorSize::Quad, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x4),
            _ => Err(ShaderError::TypeMismatch(
                "Unsupported vector type for vertex input".into(),
            )),
        },
        // Matrices or other types are not supported as direct vertex attributes in this first pass
        _ => Err(ShaderError::TypeMismatch(
            "Unsupported vertex input type".into(),
        )),
    }
}

pub(super) fn build_wgsl_from_vertex(v: &Vertex) -> String {
    // Position: location 0, vec2<f32> or vec3<f32>
    let pos_ty = if v.dimensions <= 2 {
        "vec2<f32>"
    } else {
        "vec3<f32>"
    };

    // Optional color: four floats or four plain numbers. It sits at @location(1)
    // whatever its place among the vertex's properties, since the mesh feeds it by name.
    let mut color_decl: Option<(u32, &'static str)> = None;
    if let Some(val) = v.properties.get("color")
        && matches!(val, VertexData::F32x4(_) | VertexData::Number4(_))
    {
        color_decl = Some((1, "vec4<f32>"));
    }

    let mut vs_inputs: Vec<String> = Vec::new();
    vs_inputs.push(format!("@location(0) pos: {}", pos_ty));
    if let Some((loc, ty)) = color_decl {
        vs_inputs.push(format!("@location({}) color: {}", loc, ty));
    }
    let vs_params = vs_inputs.join(", ");

    // VOut struct and assignments
    let mut vout_fields: Vec<String> = vec!["@builtin(position) pos: vec4<f32>".into()];
    let mut vs_assigns: Vec<String> = Vec::new();
    // position assignment
    if v.dimensions <= 2 {
        vs_assigns.push("out.pos = vec4<f32>(pos, 0.0, 1.0);".into());
    } else {
        vs_assigns.push("out.pos = vec4<f32>(pos, 1.0);".into());
    }

    let mut fs_return = "vec4<f32>(1.0,1.0,1.0,1.0)".to_string();
    if color_decl.is_some() {
        vout_fields.push("@location(0) color: vec4<f32>".into());
        vs_assigns.push("out.color = color;".into());
        fs_return = "v.color".into();
    }

    let vout_struct = format!("struct VOut {{ {} }};", vout_fields.join(", "));
    let vs_src = format!(
        "{vout}\n@vertex\nfn vs_main({params}) -> VOut {{\n  var out: VOut;\n  {assigns}\n  return out;\n}}",
        vout = vout_struct,
        params = vs_params,
        assigns = vs_assigns.join("\n  ")
    );
    let fs_src = format!(
        "@fragment\nfn fs_main(v: VOut) -> @location(0) vec4<f32> {{ return {ret}; }}",
        ret = fs_return
    );

    format!("{vs}\n{fs}", vs = vs_src, fs = fs_src)
}
