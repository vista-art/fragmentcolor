use super::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER};
use crate::{ShaderError, ShaderObject};

pub(super) fn load_shader(source: &str) -> Result<ShaderObject, ShaderError> {
    if source.len() < 6 {
        return Err(ShaderError::ParseError("Invalid shader source".into()));
    }

    let ext = &source[source.len() - 5..];
    let is_glsl = ext == ".glsl" || ext == ".frag" || ext == ".vert";

    let body = if source.starts_with("https:") {
        ureq::get(source).call()?.body_mut().read_to_string()? // @TODO make network calls from JS with web_sys::Request
    } else if ext == ".wgsl" || is_glsl {
        std::fs::read_to_string(source)?
    } else {
        source.to_string()
    };

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
