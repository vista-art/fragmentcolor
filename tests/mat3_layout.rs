#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader, Target};

const COLUMNS: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

const VERTEX: &str = r#"
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
"#;

async fn first_pixel(renderer: &Renderer, shader: &Shader) -> Vec<u8> {
    let target = renderer
        .create_texture_target([4, 4])
        .await
        .expect("create_texture_target failed");
    renderer
        .render(&Pass::from_shader("mat3", shader), &target)
        .expect("render failed");
    target.get_image().await[..4].to_vec()
}

// Story: A mat3x3 uniform reaches the GPU as three padded columns and reads back as 9 values.
#[test]
fn mat3_uniform_draws_its_third_column() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let source = format!(
            "{VERTEX}
@group(0) @binding(0) var<uniform> m: mat3x3<f32>;
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(m[2], 1.0); }}"
        );
        let shader = Shader::new(&source).expect("shader failed");

        // Act
        shader.set("m", COLUMNS).expect("set m");
        let pixel = first_pixel(&renderer, &shader).await;
        let m: [[f32; 3]; 3] = shader.get("m").expect("get m");

        // Assert
        assert_eq!(pixel, [0, 0, 255, 255], "m[2] is the blue column");
        assert_eq!(m, COLUMNS);
    });
}

// Story: A mat3x3 struct field keeps its columns 16 bytes apart, and the next field follows it.
#[test]
fn mat3_struct_field_draws_its_second_column() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let source = format!(
            "{VERTEX}
struct U {{ m: mat3x3<f32>, tint: vec4<f32> }};
@group(0) @binding(0) var<uniform> u: U;
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(u.m[1], 1.0) * u.tint; }}"
        );
        let shader = Shader::new(&source).expect("shader failed");

        // Act
        shader.set("u.m", COLUMNS).expect("set u.m");
        shader
            .set("u.tint", [1.0, 1.0, 1.0, 1.0])
            .expect("set u.tint");
        let pixel = first_pixel(&renderer, &shader).await;
        let m: [[f32; 3]; 3] = shader.get("u.m").expect("get u.m");
        let tint: [f32; 4] = shader.get("u.tint").expect("get u.tint");

        // Assert
        assert_eq!(pixel, [0, 255, 0, 255], "u.m[1] is the green column");
        assert_eq!(m, COLUMNS);
        assert_eq!(tint, [1.0, 1.0, 1.0, 1.0]);
    });
}

// Story: A mat3x3 inside a storage buffer uses the same padded columns.
#[test]
fn mat3_storage_field_draws_its_third_column() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let source = format!(
            "{VERTEX}
struct S {{ m: mat3x3<f32> }};
@group(0) @binding(0) var<storage, read> s: S;
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(s.m[2], 1.0); }}"
        );
        let shader = Shader::new(&source).expect("shader failed");

        // Act
        shader.set("s.m", COLUMNS).expect("set s.m");
        let pixel = first_pixel(&renderer, &shader).await;
        let m: [[f32; 3]; 3] = shader.get("s.m").expect("get s.m");

        // Assert
        assert_eq!(pixel, [0, 0, 255, 255], "s.m[2] is the blue column");
        assert_eq!(m, COLUMNS);
    });
}
