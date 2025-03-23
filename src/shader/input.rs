use super::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER};
use crate::{ShaderError, ShaderObject};

pub(super) fn load_shader(source: &str) -> Result<ShaderObject, ShaderError> {
    if source.len() < 6 {
        return Err(ShaderError::ParseError("Invalid shader source".into()));
    }

    let ext = &source[source.len() - 5..];
    let is_glsl = ext == ".glsl" || ext == ".frag" || ext == ".vert";
    let is_json = ext == ".json";

    let body = if source.starts_with("https:") {
        #[cfg(wasm)]
        return Err(ShaderError::WasmError(
            "HTTP requests in the constructor are not supported in WASM. Use Shader.fetch() instead.".into(),
        ));

        #[cfg(not(wasm))]
        ureq::get(source).call()?.body_mut().read_to_string()?
    } else if ext == ".wgsl" || is_glsl || is_json {
        std::fs::read_to_string(source)?
    } else {
        source.to_string()
    };

    // @TODO define JSON schema to extract and set default Uniform values
    if is_json {
        let json: serde_json::Value = serde_json::from_str(&body)?;
        let source = json["source"]
            .as_str()
            .ok_or_else(|| ShaderError::ParseError("JSON shader source not found".into()))?;
        return load_shader(source);
    }

    let shader_object = if ext == ".wgsl" {
        ShaderObject::wgsl(&body)?
    } else if is_glsl {
        if ext == ".vert" {
            ShaderObject::glsl(DEFAULT_VERTEX_SHADER, &body)?
        } else {
            ShaderObject::glsl(&body, DEFAULT_FRAGMENT_SHADER)?
        }
    } else {
        ShaderObject::new(&body)?
    };

    Ok(shader_object)
}
