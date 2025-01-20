use fragmentcolor;

// use winit::event_loop::EventLoop;

struct Shader {
    vertex: String,
    fragment: String,
    compute: Option<String>,
}

impl Default for Shader {
    fn default() -> Self {
        Self {
            vertex: r#"
                #version 450
                layout(location = 0) in vec2 position;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            "#
            .to_string(),
            fragment: r#"
                #version 450
                layout(location = 0) out vec4 outColor;
                void main() {
                    outColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "#
            .to_string(),
            compute: None,
        }
    }
}

fn main() {
    // let event_loop = EventLoop::new().unwrap();

    let color = fragmentcolor::Color::from_hex("#ff0000").unwrap();

    // Hello triangle shader source
    let shader = Shader {
        vertex: r#"
            #version 450
            layout(location = 0) in vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#
        .to_string(),
        fragment: r#"
            #version 450
            layout(location = 0) out vec4 outColor;
            void main() {
                outColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#
        .to_string(),
        compute: None,
    };

    println!("Color: {:?}", color);
}
