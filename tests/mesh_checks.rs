#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{
    Instance, Material, Mesh, MeshError, Model, Pass, Renderer, RendererError, Shader,
    ShaderError, Target, Vertex,
};

const WHITE: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

/// Draws white, moved by each instance's `offset`.
const OFFSET: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) offset: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position + offset, 0.0, 1.0);
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

// Story: a mesh without set_indices draws its vertices in threes, so a count
// that ends partway through a triangle fails render, where the GPU dropped the rest.
#[test]
fn vertices_without_indices_come_in_threes() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(WHITE).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(Vertex::new));
        mesh.add_vertex(Vertex::new([0.5, 0.5]));
        shader.add_mesh(&mesh).expect("attach");

        let partial = renderer.render(&shader, &target);
        mesh.set_indices([0, 1, 2]);
        let indexed = renderer.render(&shader, &target);

        assert_eq!(
            mesh_error(partial),
            "A mesh without set_indices draws its vertices in threes, and it holds 4"
        );
        assert!(indexed.is_ok(), "got {indexed:?}");
    });
}

// Story: Shader::from_mesh fails for an empty mesh and for a mesh its shader
// refuses, where it returned a shader that drew nothing or the default image.
#[test]
fn from_mesh_fails_for_a_mesh_it_cannot_draw() {
    let empty = Shader::from_mesh(&Mesh::new());
    let mixed = Mesh::new();
    mixed.add_vertices([
        Vertex::new([0.0, 0.0]),
        Vertex::new([1.0, 0.0, 0.0]),
        Vertex::new([0.0, 1.0]),
    ]);
    let refused = Shader::from_mesh(&mixed);
    let good = Mesh::new();
    good.add_vertices(FULLSCREEN.map(Vertex::new));
    let drawn = Shader::from_mesh(&good);

    match empty {
        Err(ShaderError::InvalidKey(message)) => assert_eq!(
            message,
            "Mesh has no vertices, and Shader::from_mesh builds a shader from the first one"
        ),
        other => panic!("expected an invalid key, got {:?}", other.err()),
    }
    match refused {
        Err(ShaderError::AttributeMismatch { name, .. }) => assert_eq!(name, "position"),
        other => panic!("expected an attribute mismatch, got {:?}", other.err()),
    }
    assert!(drawn.is_ok_and(|shader| shader.list_uniforms().is_empty()));
}

// Story: the mesh quick view returns the reason its attach failed, where render
// said its shader had no mesh to read from.
#[test]
fn the_quick_view_reports_its_attach_error() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let mesh = Mesh::new();
        mesh.add_vertices(
            FULLSCREEN.map(|position| Vertex::new(position).set("position", [5.0, 5.0])),
        );

        let rendered = renderer.render(&mesh, &target);
        let again = renderer.render(&mesh, &target);

        for result in [rendered, again] {
            match result {
                Err(RendererError::ShaderError(ShaderError::InvalidKey(message))) => {
                    assert!(message.contains("property named 'position'"), "{message}")
                }
                other => panic!("expected the attach error, got {other:?}"),
            }
        }
    });
}

// Story: an upload that fails leaves the vertices it packed dirty, so the render
// after the fix draws the new triangle, where it drew the old buffers.
#[test]
fn a_failed_upload_writes_its_vertices_next_time() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 2])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(OFFSET).expect("shader failed");
        let mesh = Mesh::new();
        // The left half, then the right half, as two pairs of triangles drawn by index.
        mesh.add_vertices([[-1.0, -1.0], [0.0, -1.0], [-1.0, 3.0]].map(Vertex::new));
        mesh.set_indices([0, 1, 2]);
        mesh.add_instance(Instance::new().set("offset", [0.0, 0.0]));
        shader.add_mesh(&mesh).expect("attach");
        renderer.render(&shader, &target).expect("first render");

        mesh.add_vertices([[0.0, -1.0], [1.0, -1.0], [1.0, 3.0]].map(Vertex::new));
        mesh.set_indices([3, 4, 5]);
        mesh.add_instance(Instance::new());
        let refused = renderer.render(&shader, &target);
        mesh.clear_instances();
        mesh.add_instance(Instance::new().set("offset", [0.0, 0.0]));
        let fixed = renderer.render(&shader, &target);
        let image = target.get_image().await.expect("read back");

        assert!(refused.is_err(), "an instance without offset is refused");
        assert!(fixed.is_ok(), "got {fixed:?}");
        assert_eq!(image[4..8], [0, 0, 0, 0], "the old triangle is gone");
        assert_eq!(
            image[28..32],
            [255, 255, 255, 255],
            "the new triangle draws"
        );
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

/// A Model of `material` on `corners`, drawn without indices.
fn model_on(material: &Material, corners: &[[f32; 3]]) -> Model {
    let mesh = Mesh::new();
    mesh.add_vertices(corners.iter().map(|corner| Vertex::pbr(*corner)));
    Model::new(mesh, material.clone())
}

const TRIANGLE: [[f32; 3]; 3] = [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]];
const EDGE: [[f32; 3]; 2] = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];

// Story: a frame checks only the meshes it draws. A hidden Model, or a Model of
// another pass, can hold a mesh that is still taking its vertices, and the visible
// Models that share its material draw. A visible partial mesh still fails.
#[test]
fn a_frame_checks_only_the_meshes_it_draws() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let material = Material::pbr();
        let floor = model_on(&material, &TRIANGLE);
        let growing = model_on(&material, &EDGE);
        growing.set_visible(false);

        let shared = Pass::new("shared");
        shared.add(&floor).expect("floor");
        shared.add(&growing).expect("growing");
        let hidden = renderer.render(&shared, &target);

        let alone = Pass::new("alone");
        alone.add(&growing).expect("growing alone");
        let skipped = renderer.render(&alone, &target);

        let elsewhere = Pass::new("elsewhere");
        elsewhere
            .add(&model_on(&material, &EDGE))
            .expect("partial");
        let other = renderer.render(&shared, &target);

        growing.set_visible(true);
        let visible = mesh_error(renderer.render(&shared, &target));

        assert!(hidden.is_ok(), "a hidden partial Model: {hidden:?}");
        assert!(skipped.is_ok(), "a pass of hidden Models only: {skipped:?}");
        assert!(other.is_ok(), "a partial Model of another pass: {other:?}");
        assert_eq!(visible, MeshError::VertexCount(2).to_string());
    });
}
