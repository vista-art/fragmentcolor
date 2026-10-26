use super::create_validator;
use crate::{ShaderError, ShaderObject};

impl ShaderObject {
    /// Create a Shader object from a GLSL source pair (vertex and fragment shaders).
    pub fn glsl(vertex_source: &str, fragment_source: &str) -> Result<Self, ShaderError> {
        use naga::back::wgsl;
        use naga::front::glsl;

        let mut parser = glsl::Frontend::default();
        let mut validator = create_validator();

        let wgsl_vertex_source = {
            let vertex_module = parser.parse(
                &glsl::Options::from(naga::ShaderStage::Vertex),
                vertex_source,
            )?;
            let vertex_module_info = validator.validate(&vertex_module).map_err(Box::new)?;

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
            let fragment_module_info = validator.validate(&fragment_module).map_err(Box::new)?;

            wgsl::write_string(
                &fragment_module,
                &fragment_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn fs_main")
        };

        Self::wgsl(&format!("{}\n{}", wgsl_vertex_source, wgsl_fragment_source))
    }
}
