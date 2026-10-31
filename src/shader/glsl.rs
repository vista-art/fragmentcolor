use super::error::{Diagnostic, Source, WriteFailure};
use super::validate_module;
use crate::{ShaderError, ShaderObject};
use naga::back::wgsl;
use naga::front::glsl;

impl ShaderObject {
    /// Create a Shader object from a GLSL source pair (vertex and fragment shaders).
    pub fn glsl(vertex_source: &str, fragment_source: &str) -> Result<Self, ShaderError> {
        let vertex = convert_stage(vertex_source, naga::ShaderStage::Vertex, "vertex.glsl")?;
        let fragment = convert_stage(
            fragment_source,
            naga::ShaderStage::Fragment,
            "fragment.glsl",
        )?;
        let vertex = vertex.replace("fn main", "fn vs_main");
        let fragment = fragment.replace("fn main", "fn fs_main");
        Self::wgsl(&format!("{vertex}\n{fragment}"))
    }
}

/// The WGSL for the GLSL `text` of one `stage`, or naga's report on a source its parser,
/// validator or writer refuses, which names the stage's source as `path`.
fn convert_stage(
    text: &str,
    stage: naga::ShaderStage,
    path: &'static str,
) -> Result<String, ShaderError> {
    let source = Source { text, path };
    let module = match glsl::Frontend::default().parse(&glsl::Options::from(stage), text) {
        Ok(module) => module,
        Err(errors) => {
            let diagnostic = Diagnostic::from_glsl(&errors, source);
            return Err(ShaderError::GlslParseErrors(diagnostic));
        }
    };
    let info = validate_module(&module, Some(source))?;
    Ok(write_stage(&module, &info, source)?)
}

/// The WGSL naga writes for one stage's module, or the failure that names the stage's source.
fn write_stage<'text>(
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
    source: Source<'text>,
) -> Result<String, WriteFailure<'text>> {
    match wgsl::write_string(module, info, wgsl::WriterFlags::empty()) {
        Ok(text) => Ok(text),
        Err(error) => Err(WriteFailure { source, error }),
    }
}
