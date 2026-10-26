#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Mesh, Pass, Renderer, Shader, Target, Vertex};

const WHITE: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

const FULLSCREEN: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];

// Story: a mesh draws with a second Renderer after the first one drew it, and
// with the first one again, each from buffers on its own device, where wgpu panicked.
#[test]
fn a_mesh_draws_with_two_renderers() {
    pollster::block_on(async move {
        let shader = Shader::new(WHITE).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(Vertex::new));
        shader.add_mesh(&mesh).expect("attach");
        let pass = Pass::from_shader("two renderers", &shader);
        let mut renderers = Vec::new();
        for _ in 0..2 {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("create_texture_target failed");
            renderers.push((renderer, target));
        }

        let mut pixels = Vec::new();
        for index in [0, 1, 0] {
            let (renderer, target) = &renderers[index];
            renderer.render(&pass, target).expect("render");
            pixels.push(target.get_image().await.expect("read back")[..4].to_vec());
        }

        let white = [255, 255, 255, 255];
        assert_eq!(pixels[0], white, "the first renderer");
        assert_eq!(pixels[1], white, "the second renderer");
        assert_eq!(pixels[2], white, "the first renderer again");
    });
}
