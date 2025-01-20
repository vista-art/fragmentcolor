struct Shader {
    id: u32,
    source: String,
    uniforms: HashMap<String, Uniform>,
}

/// I want a ShaderInput that can be serialized and saved in a database,
/// Then easily shared between many programming languages. This can be JSON
/// including the shader source and the uniforms.

struct ShaderInput {
    source: String,
    uniforms: HashMap<String, Uniform>,
}

/// I want some magic so users can simply input a string with the Shader Source
/// and the Uniforms would be inferred from it.


/// Example:
/// ```rust
/// 
/// 
/// 
/// let mut shader = Shader::new("
///    void main() {
///       gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
///   }
/// ");
/// 
/// 
/// shader.uniform("iResolution", [800.0, 600.0]);
/// ```
/// 




impl Shader {
    pub fn new(source: &str) -> Self {
        Self {
            id: 0,
            source: source.to_string(),
            uniforms: HashMap::new(),
        }
    }

    pub fn uniform(&mut self, name: &str, value: Uniform) {
        self.uniforms.insert(name.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader() {
        let shader = Shader::new("shader");
        assert_eq!(shader.source, "shader");
    }
}
