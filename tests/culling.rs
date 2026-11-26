#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Mesh, Model, Pass, Renderer, Shader, Target};

/// A white triangle in clip space, moved by the Model columns.
const MODEL_SHADER: &str = r#"
@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(3) model_0: vec4<f32>,
  @location(4) model_1: vec4<f32>,
  @location(5) model_2: vec4<f32>,
  @location(6) model_3: vec4<f32>,
) -> @builtin(position) vec4<f32> {
  return mat4x4<f32>(model_0, model_1, model_2, model_3) * vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

/// A white triangle in clip space, drawn from its mesh as it is.
const MESH_SHADER: &str = r#"
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

/// A triangle around the center that winds clockwise, so a viewer sees its back.
fn back_facing() -> Mesh {
    let mesh = Mesh::new();
    mesh.add_vertices([[0.0, 0.8, 0.0], [0.8, -0.8, 0.0], [-0.8, -0.8, 0.0]]);
    mesh
}

/// Renders `pass` into a 16x16 target cleared to black and returns the red byte at its center.
async fn center(renderer: &Renderer, pass: &Pass) -> u8 {
    let target = renderer
        .create_texture_target([16, 16])
        .await
        .expect("texture target");
    renderer.render(pass, &target).expect("render");
    let image = target.get_image().await.expect("read back");
    image[(8 * 16 + 8) * 4]
}

/// A pass that draws the back-facing triangle as a Model with `shader` as its material.
fn as_model(shader: Shader) -> Pass {
    let pass = Pass::new("model");
    pass.add(&Model::new(back_facing(), shader))
        .expect("add the model");
    pass
}

/// A pass that draws the back-facing triangle as a mesh of `shader`.
fn as_mesh(shader: Shader) -> Pass {
    shader.add_mesh(&back_facing()).expect("attach the mesh");
    Pass::from_shader("mesh", &shader)
}

// Story: every Shader culls back faces, as the materials do, whether it draws a
// Model or a mesh of its own. `double_sided(true)` draws both faces.
#[test]
fn every_shader_culls_back_faces_until_double_sided() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let model = || Shader::new(MODEL_SHADER).expect("model shader");
        let mesh = || Shader::new(MESH_SHADER).expect("mesh shader");

        // Act
        let model_culled = center(&renderer, &as_model(model())).await;
        let model_both = center(&renderer, &as_model(model().double_sided(true))).await;
        let mesh_culled = center(&renderer, &as_mesh(mesh())).await;
        let mesh_both = center(&renderer, &as_mesh(mesh().double_sided(true))).await;

        // Assert
        assert_eq!(model_culled, 0, "a Model's Shader culls the back face");
        assert_eq!(model_both, 255, "double_sided(true) draws it on a Model");
        assert_eq!(mesh_culled, 0, "a Shader's own mesh culls the back face");
        assert_eq!(mesh_both, 255, "double_sided(true) draws it on a mesh");
    });
}

// Story: the default Shader's triangle and a fullscreen triangle wind
// counter-clockwise, so they face the viewer and survive culling.
#[test]
fn the_default_shader_and_a_fullscreen_triangle_face_the_viewer() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let triangle = Pass::from_shader("default", &Shader::default());
        let fullscreen = Shader::new(
            r#"
            @vertex
            fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
              let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
              return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
            }

            @fragment
            fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
            "#,
        )
        .expect("fullscreen shader");
        let fullscreen = Pass::from_shader("fullscreen", &fullscreen);

        // Act
        let triangle = center(&renderer, &triangle).await;
        let fullscreen = center(&renderer, &fullscreen).await;

        // Assert
        assert_eq!(triangle, 255, "the default red triangle draws");
        assert_eq!(fullscreen, 255, "the fullscreen triangle draws");
    });
}
