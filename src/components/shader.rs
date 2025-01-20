#[derive(Clone, Default, Debug)]
pub struct Shader {
    pub source: String,
}

// @TODO - the renderpass should read from here
//       - The EventLoop should watch when they change the source

/// Adding a custom shader to an Object is just a matter of adding
/// `my_object.add_component(Shader::new("my_shader.glsl"))` or
/// `my_object.add_component(Shader::new(include_str!("my_shader.glsl")))`.
impl Shader {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }
}
