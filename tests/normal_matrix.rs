#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader};

// Two normals through two models: one scaled 2x along x, one mirrored along x.
const NORMALS: &str = r#"
@group(0) @binding(0) var<storage, read_write> normals: array<vec4<f32>, 3>;

@compute @workgroup_size(1)
fn cs_main() {
  let stretched = mat4x4<f32>(
    vec4<f32>(2.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
  );
  let mirrored = mat4x4<f32>(
    vec4<f32>(-1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
  );
  let slope = normalize(vec3<f32>(1.0, 1.0, 0.0));
  normals[0] = vec4<f32>(mesh_transform_normal(slope, stretched), 0.0);
  normals[1] = vec4<f32>(mesh_transform_normal(vec3<f32>(1.0, 0.0, 0.0), mirrored), 0.0);
  normals[2] = vec4<f32>(mesh_transform_normal(vec3<f32>(0.0, 1.0, 0.0), mirrored), 0.0);
}
"#;

fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

fn assert_near(found: &[f32], expected: [f32; 3], label: &str) {
    for (axis, (found, expected)) in found.iter().zip(expected).enumerate() {
        assert!(
            (found - expected).abs() < 1e-5,
            "{label}: axis {axis} is {found}, expected {expected}"
        );
    }
}

// Story: a normal follows the inverse transpose of the model, so a surface
// scaled unevenly or mirrored still lights the way it faces.
#[test]
fn normals_follow_uneven_scale_and_mirroring() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([1, 1])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(["mesh/transform", NORMALS]).expect("shader failed");
        let pass = Pass::from_shader("normals", &shader);

        // Act
        renderer.render(&pass, &probe).expect("render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "normals")
            .await
            .expect("read_storage failed");
        let normals = floats(&bytes);
        // A 45° slope stretched 2x along x leans toward y: (0.5, 1, 0) normalized.
        let lean = 1.0 / 1.25_f32.sqrt();
        assert_near(&normals[0..3], [0.5 * lean, lean, 0.0], "stretched slope");
        assert_near(&normals[4..7], [-1.0, 0.0, 0.0], "mirrored side");
        assert_near(&normals[8..11], [0.0, 1.0, 0.0], "mirrored top");
    });
}
