use crate::{ShaderError, ShaderObject};

impl ShaderObject {
    #[cfg(not(feature = "glsl"))]
    pub fn glsl(_vertex_source: &str, _fragment_source: &str) -> Result<Self, ShaderError> {
        Err(ShaderError::ParseError("GLSL is not enabled".into()))
    }

    #[cfg(feature = "glsl")]
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
                .validate(&vertex_module)?;

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
                .validate(&fragment_module)?;

            wgsl::write_string(
                &fragment_module,
                &fragment_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn fs_main")
        };

        Self::wgsl(&format!("{}\n{}", wgsl_vertex_source, wgsl_fragment_source))
    }

    #[cfg(feature = "shadertoy")]
    /// Create a Shader object from a Shadertoy-flavored GLSL source.
    pub fn toy(source: &str) -> Result<Self, ShaderError> {
        use crate::DEFAULT_VERTEX_SHADER;
        use crate::SHADERTOY_WRAPPER;

        Self::glsl(
            DEFAULT_VERTEX_SHADER,
            &SHADERTOY_WRAPPER.replace("{{shader}}", source),
        )
    }
}
