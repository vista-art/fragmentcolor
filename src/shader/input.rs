use super::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER};
use crate::ShaderObject;
use crate::shader::error::ShaderError;
use std::sync::Arc;

pub(super) fn load_shader(source: &str) -> Result<Arc<ShaderObject>, ShaderError> {
    if source.is_empty() {
        return Ok(Arc::new(ShaderObject::default()));
    }

    if source.len() < 6 {
        return Err(ShaderError::ParseError("Invalid shader source".into()));
    }

    let ext = &source[source.len() - 5..];
    let is_glsl = ext == ".glsl" || ext == ".frag" || ext == ".vert";

    let body = if source.starts_with("https:") {
        #[cfg(wasm)]
        return Err(ShaderError::Error(
            "HTTP requests in the constructor are not supported in WASM. Use await Shader.fetch() instead.".into(),
        ));

        #[cfg(not(wasm))]
        ureq::get(source).call()?.body_mut().read_to_string()?
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

    Ok(Arc::new(shader_object))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Load minimal WGSL from a file path with .wgsl extension.
    #[test]
    fn loads_minimal_wgsl_from_file() {
        let wgsl = r#"
@group(0) @binding(0) var<uniform> u: vec4<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i) - 1);
  let y = f32(i32(i & 1u) * 2 - 1);
  return vec4<f32>(x, y, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 1.0, 1.0, 1.0); }
        "#;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("min.wgsl");
        std::fs::write(&path, wgsl).expect("write");
        let res = load_shader(path.to_str().unwrap());
        assert!(res.is_ok());
    }

    // Story: Invalid GLSL produces error
    #[test]
    fn glsl_file_without_feature_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("m.frag");
        std::fs::write(&path, "void main() {}").expect("write");
        let res = load_shader(path.to_str().unwrap());

        // With GLSL enabled, invalid GLSL should still error (validation error path)
        assert!(res.is_err());
    }

    // Story: Very short source string is rejected as invalid shader source.
    #[test]
    fn short_source_string_rejected() {
        let err = load_shader("x").expect_err("invalid short source");
        match err {
            ShaderError::ParseError(_) => {}
            _ => panic!("unexpected error kind: {:?}", err),
        }
    }
}
