#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Mesh, Pass, Renderer, RendererError, Shader, ShaderError, Target, Vertex};

const WHITE: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

const FULLSCREEN: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];

/// The message of the mesh error `result` holds.
fn mesh_error(result: Result<(), RendererError>) -> String {
    match result {
        Err(RendererError::MeshError(error)) => error.to_string(),
        other => panic!("expected a mesh error, got {other:?}"),
    }
}

// Story: indices from set_indices form whole triangles of vertices the mesh
// holds, or render fails, where the GPU read zeros or dropped the extra index.
#[test]
fn indices_point_at_whole_triangles() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(WHITE).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(Vertex::new));
        shader.add_mesh(&mesh).expect("attach");

        mesh.set_indices([0, 1, 5]);
        let past = renderer.render(&shader, &target);
        mesh.set_indices([0, 1, 2, 0]);
        let partial = renderer.render(&shader, &target);
        mesh.set_indices([0, 1, 2]);
        let whole = renderer.render(&shader, &target);

        assert_eq!(
            mesh_error(past),
            "Index 5 points past the last of 3 vertices"
        );
        assert_eq!(
            mesh_error(partial),
            "A triangle list takes a multiple of 3 indices, got 4"
        );
        assert!(whole.is_ok(), "got {whole:?}");
    });
}

// Story: a vertex takes its position from Vertex::new, so a property named
// position fails the attach, where the value set never reached the GPU.
#[test]
fn position_is_no_property() {
    let shader = Shader::new(WHITE).expect("shader failed");
    let mesh = Mesh::new();
    mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("position", [5.0, 5.0])));

    let attached = shader.add_mesh(&mesh);

    match attached {
        Err(ShaderError::InvalidKey(message)) => assert_eq!(
            message,
            "vertex 0 sets a property named 'position', and a vertex takes its position from Vertex::new"
        ),
        other => panic!("expected an invalid key, got {other:?}"),
    }
}

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
