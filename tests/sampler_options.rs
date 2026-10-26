#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, SamplerOptions, Shader, Target, Texture, TextureOptions};

// Samples a 2x1 texture at `(u, 0.5)` and paints the whole target with it.
const SAMPLE_AT_U: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var<uniform> u: f32;

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}

@fragment fn fs_main() -> @location(0) vec4<f32> {
  return textureSampleLevel(tex, smp, vec2<f32>(u, 0.5), 0.0);
}
"#;

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

const RED_BLUE: &[u8] = &[255, 0, 0, 255, 0, 0, 255, 255];

async fn red_blue_texture(renderer: &Renderer) -> Texture {
    renderer
        .create_texture((RED_BLUE, [2, 1]))
        .await
        .expect("create_texture failed")
}

// A 2x1 size with a nearest sampler that repeats along x.
fn repeat_x_options() -> TextureOptions {
    TextureOptions {
        size: Some([2, 1].into()),
        sampler: SamplerOptions {
            repeat_x: true,
            smooth: false,
            ..Default::default()
        },
        ..Default::default()
    }
}

async fn sample(renderer: &Renderer, shader: &Shader, u: f32) -> Vec<u8> {
    let target = renderer
        .create_texture_target([4, 4])
        .await
        .expect("create_texture_target failed");
    shader.set("u", u).expect("set u");
    renderer
        .render(&Pass::from_shader("sample", shader), &target)
        .expect("render failed");
    target.get_image().await.expect("read back")[..4].to_vec()
}

// Story: Turning on repeat makes the bound sampler wrap past the edge instead of clamping.
#[test]
fn repeat_wraps_past_the_edge() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let texture = red_blue_texture(&renderer).await;
        let shader = Shader::new(SAMPLE_AT_U).expect("shader failed");
        shader.set("tex", &texture).expect("set tex");

        // Act
        let clamped = sample(&renderer, &shader, 1.25).await;
        texture.set_sampler_options(SamplerOptions {
            repeat_x: true,
            repeat_y: true,
            smooth: false,
            compare: None,
        });
        let wrapped = sample(&renderer, &shader, 1.25).await;

        // Assert
        assert_eq!(clamped, BLUE, "clamped: u = 1.25 reads the last texel");
        assert_eq!(wrapped, RED, "repeated: u = 1.25 wraps to the first texel");
    });
}

// Story: Sampler options passed to create_texture wrap the texture from its first render.
#[test]
fn creation_options_wrap_past_the_edge() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let texture = renderer
            .create_texture((RED_BLUE, repeat_x_options()))
            .await
            .expect("create_texture failed");
        let shader = Shader::new(SAMPLE_AT_U).expect("shader failed");
        shader.set("tex", &texture).expect("set tex");

        // Act
        let wrapped = sample(&renderer, &shader, 1.25).await;

        // Assert
        assert_eq!(wrapped, RED, "repeated: u = 1.25 wraps to the first texel");
    });
}

// Story: Sampler options passed to create_storage_texture wrap the texture from its first render.
#[test]
fn storage_creation_options_wrap_past_the_edge() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let texture = renderer
            .create_storage_texture((RED_BLUE, repeat_x_options()))
            .await
            .expect("create_storage_texture failed");
        let shader = Shader::new(SAMPLE_AT_U).expect("shader failed");
        shader.set("tex", &texture).expect("set tex");

        // Act
        let wrapped = sample(&renderer, &shader, 1.25).await;

        // Assert
        assert_eq!(wrapped, RED, "repeated: u = 1.25 wraps to the first texel");
    });
}

// Story: Turning smooth off makes the bound sampler pick one texel instead of blending two.
#[test]
fn smooth_blends_neighboring_texels() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let texture = red_blue_texture(&renderer).await;
        let shader = Shader::new(SAMPLE_AT_U).expect("shader failed");
        shader.set("tex", &texture).expect("set tex");

        // Act
        let blended = sample(&renderer, &shader, 0.6).await;
        texture.set_sampler_options(SamplerOptions {
            smooth: false,
            ..Default::default()
        });
        let nearest = sample(&renderer, &shader, 0.6).await;

        // Assert
        assert!(
            blended[0] > 0 && blended[2] > blended[0],
            "smooth: u = 0.6 mixes red into blue, got {blended:?}"
        );
        assert_eq!(nearest, BLUE, "nearest: u = 0.6 reads the second texel");
    });
}

// Story: A depth texture owns a comparison sampler, and a plain sampler next to it still binds.
// The plain sampler samples a color texture: a filtering sampler never samples depth,
// which the web refuses, so `Shader::new` refuses that pair.
#[test]
fn depth_texture_binds_to_a_plain_sampler() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("create_texture_target failed");
        let depth = renderer
            .create_depth_texture([4, 4])
            .await
            .expect("create_depth_texture failed");
        let color = renderer
            .create_texture((vec![255u8; 4 * 4 * 4], [4, 4]))
            .await
            .expect("create_texture failed");
        let shader = Shader::new(
            r#"
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var image: texture_2d<f32>;

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}

@fragment fn fs_main() -> @location(0) vec4<f32> {
  let d = textureLoad(depth, vec2<i32>(0, 0), 0);
  let c = textureSampleLevel(image, smp, vec2<f32>(0.5, 0.5), 0.0);
  return vec4<f32>(c.rgb * d, 1.0);
}
"#,
        )
        .expect("shader failed");
        shader.set("depth", &depth).expect("set depth");
        shader.set("image", &color).expect("set image");

        // Act
        let result = renderer.render(&Pass::from_shader("depth", &shader), &target);

        // Assert
        assert!(result.is_ok(), "render failed: {result:?}");
    });
}
