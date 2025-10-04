use crate::{ShaderError, ShaderObject};

impl ShaderObject {
    /// Create a Shader object from a GLSL source pair (vertex and fragment shaders).
    pub fn glsl(vertex_source: &str, fragment_source: &str) -> Result<Self, ShaderError> {
        use naga::back::wgsl;
        use naga::front::glsl;
        use naga::valid::{
            Capabilities, ShaderStages, SubgroupOperationSet, ValidationFlags, Validator,
        };

        let mut parser = glsl::Frontend::default();
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());

        let wgsl_vertex_source = {
            let vertex_module = parser.parse(
                &glsl::Options::from(naga::ShaderStage::Vertex),
                vertex_source,
            )?;
            let vertex_module_info = validator
                .subgroup_stages(ShaderStages::VERTEX)
                .subgroup_operations(SubgroupOperationSet::all())
                .validate(&vertex_module)
                .map_err(Box::new)?;

            wgsl::write_string(
                &vertex_module,
                &vertex_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn vs_main")
        };

        let wgsl_fragment_source = {
            let fragment_module = parser.parse(
                &glsl::Options::from(naga::ShaderStage::Fragment),
                fragment_source,
            )?;
            let fragment_module_info = validator
                .subgroup_stages(ShaderStages::FRAGMENT)
                .subgroup_operations(SubgroupOperationSet::all())
                .validate(&fragment_module)
                .map_err(Box::new)?;

            wgsl::write_string(
                &fragment_module,
                &fragment_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn fs_main")
        };

        Self::wgsl(&format!("{}\n{}", wgsl_vertex_source, wgsl_fragment_source))
    }

    // @TODO
    /// Create a Shader object from a Shadertoy-flavored GLSL source.
    pub fn _toy(source: &str) -> Result<Self, ShaderError> {
        use crate::DEFAULT_VERTEX_SHADER;
        use crate::SHADERTOY_WRAPPER;

        Self::glsl(
            DEFAULT_VERTEX_SHADER,
            &SHADERTOY_WRAPPER.replace("{{shader}}", source),
        )
    }
}
