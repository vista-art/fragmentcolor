#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader, Target};

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
        let img = target.get_image().await;
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
        target.resize([32, 8]);
        renderer.render(&pass, &scratch).expect("render failed");

        let img = target.get_image().await;
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
        target.resize([24, 24]);
        fill.set("color", [0.0, 1.0, 0.0, 1.0]).expect("set failed");
        renderer.render(&draw, &probe).expect("render failed");
        renderer.render(&post, &probe).expect("render failed");

        let img = probe.get_image().await;
        assert!(
            img[1] > 200 && img[0] < 50,
            "sampled binding should see the repainted resized texture, got {:?}",
            &img[..4]
        );
    });
}
