#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::texture::TextureError;
use fragmentcolor::{InitializationError, Pass, Renderer, Shader, Target};

const FILL: &str = r#"
struct VertexOutput { @builtin(position) position: vec4<f32> }

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var p = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    return VertexOutput(vec4<f32>(p[i], 0.0, 1.0));
}

@group(0) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> { return color; }
"#;

const SAMPLE_CENTER: &str = r#"
@group(0) @binding(0) var input_image: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4<f32>(p[i], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(input_image, input_sampler, vec2<f32>(0.5, 0.5));
}
"#;

// Story: Rendering into an offscreen texture should succeed and produce readable pixels.
#[test]
fn renders_to_texture_and_reads_back() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::default();

        // Act
        renderer.render(&shader, &target).expect("render failed");

        // Assert
        let img = target.get_image().await.expect("read back");
        assert!(!img.is_empty(), "image readback should not be empty");
    });
}

// Story: A Pass color target keeps drawing into the live texture after resize.
#[test]
fn pass_target_draws_into_resized_texture() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let scratch = renderer
            .create_texture_target([4, 4])
            .await
            .expect("create_texture_target failed");

        let fill = Shader::new(FILL).expect("fill shader failed");
        fill.set("color", [1.0, 0.0, 0.0, 1.0]).expect("set failed");
        let pass = Pass::new("fill");
        pass.add_shader(&fill);
        pass.set_target(&target).expect("set_target failed");

        renderer.render(&pass, &scratch).expect("render failed");
        target.resize([32, 8]).expect("resize failed");
        renderer.render(&pass, &scratch).expect("render failed");

        let img = target.get_image().await.expect("read back");
        assert_eq!(img.len(), 32 * 8 * 4, "readback should match the new size");
        assert!(
            img[0] > 200 && img[1] < 50 && img[3] == 255,
            "resized texture should hold the pass output, got {:?}",
            &img[..4]
        );
    });
}

// Story: A shader sampling target.texture() reads the live texture after resize.
#[test]
fn sampled_binding_reads_resized_texture() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");

        let fill = Shader::new(FILL).expect("fill shader failed");
        fill.set("color", [1.0, 0.0, 0.0, 1.0]).expect("set failed");
        let draw = Pass::new("draw");
        draw.add_shader(&fill);
        draw.set_target(&target).expect("set_target failed");

        let post = Shader::new(SAMPLE_CENTER).expect("sample shader failed");
        post.set("input_image", &target.texture())
            .expect("set failed");

        renderer.render(&draw, &probe).expect("render failed");

        // repaint at a new size, then sample through the pre-resize binding
        target.resize([24, 24]).expect("resize failed");
        fill.set("color", [0.0, 1.0, 0.0, 1.0]).expect("set failed");
        renderer.render(&draw, &probe).expect("render failed");
        renderer.render(&post, &probe).expect("render failed");

        let img = probe.get_image().await.expect("read back");
        assert!(
            img[1] > 200 && img[0] < 50,
            "sampled binding should see the repainted resized texture, got {:?}",
            &img[..4]
        );
    });
}

// Story: resizing a target's texture() handle resizes the target itself.
#[test]
fn a_texture_handle_resizes_its_target() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        target.texture().resize([24, 16]).expect("resize failed");
        let size: [u32; 2] = target.size().into();
        assert_eq!(size, [24, 16]);
        let img = target.get_image().await.expect("read back");
        assert_eq!(img.len(), 24 * 16 * 4);
    });
}

// Story: a texture() handle and a clone of the target both follow a resize,
// so they read the new size and the pixels drawn after it.
#[test]
fn handles_and_clones_follow_resize() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let handle = target.texture();
        let copy = target.clone();

        target.resize([24, 16]).expect("resize failed");
        let fill = Shader::new(FILL).expect("fill shader failed");
        fill.set("color", [1.0, 0.0, 0.0, 1.0]).expect("set failed");
        renderer.render(&fill, &target).expect("render failed");

        let size: [u32; 2] = handle.size().into();
        assert_eq!(size, [24, 16], "the handle reads the new size");
        let size: [u32; 2] = copy.size().into();
        assert_eq!(size, [24, 16], "the clone reads the new size");
        let img = handle.get_image().await.expect("read back");
        assert_eq!(img.len(), 24 * 16 * 4, "the handle reads the new texture");
        assert!(
            img[0] > 200,
            "the handle sees the fill, got {:?}",
            &img[..4]
        );
    });
}

// Story: a texture target refuses a size with no area and keeps its own,
// and the size it already has keeps its pixels.
#[test]
fn resize_refuses_an_empty_size_and_keeps_the_pixels() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let fill = Shader::new(FILL).expect("fill shader failed");
        fill.set("color", [1.0, 0.0, 0.0, 1.0]).expect("set failed");
        renderer.render(&fill, &target).expect("render failed");

        let refused = target.resize([0, 8]);
        assert!(refused.is_err(), "a width of zero is an error");
        let size: [u32; 2] = target.size().into();
        assert_eq!(size, [8, 8], "a refused resize keeps the size");

        target.resize([8, 8]).expect("resize failed");
        let img = target.get_image().await.expect("read back");
        assert!(
            img[0] > 200,
            "the current size keeps the pixels, got {:?}",
            &img[..4]
        );
    });
}

// Story: a texture target past the device limit, or with no area, is refused at
// creation with the error resize returns, before it reaches wgpu.
#[test]
fn create_texture_target_checks_the_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        for size in [[100_000, 8], [0, 8]] {
            let refused = renderer.create_texture_target(size).await;
            assert!(
                matches!(
                    refused,
                    Err(InitializationError::Texture(TextureError::InvalidInput(_)))
                ),
                "{size:?} should be refused"
            );
        }
    });
}
