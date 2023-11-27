use crate::scene::macros::api_object;
use crate::{Border, Bounds, Color, Object, Quad, Renderable2D, SceneObject, ShapeFlag};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Shader(String);

// @TODO - the renderpass should read from here
//       - The EventLoop should watch when they change the source

/// Adding a custom shader to an Object is just a matter of adding
/// `my_object.add_component(Shader::new("my_shader.glsl"))` or
/// `my_object.add_component(Shader::new(include_str!("my_shader.glsl")))`.
impl Shader {
    pub fn new(source: &str) -> Object<Self> {
        let mut shader = Object::new(Self(source.to_string()));

        let components = Renderable2D {
            transform: shader.transform_id(),
            image: None,
            bounds: Bounds(Quad::default()),
            color: Color::default(),
            border: Border(0.0),
            sdf_flags: ShapeFlag(99.0),
        };

        shader.add_components(components);

        shader
    }
}

api_object!(Shader);
