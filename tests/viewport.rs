#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, PassError, Renderer, ScreenRegion, Shader, Target};

const FILL: &str = r#"
@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4<f32>(p[i], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> { return color; }
"#;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

/// A pass that fills its viewport with `color` over a blue clear.
fn fill_pass<R>(viewport: R, color: [f32; 4]) -> Pass
where
    R: TryInto<ScreenRegion>,
    PassError: From<R::Error>,
{
    let shader = Shader::new(FILL).expect("fill shader failed");
    shader.set("color", color).expect("set failed");
    let pass = Pass::from_shader("fill", &shader);
    pass.set_clear_color([0.0, 0.0, 1.0, 1.0])
        .expect("set_clear_color failed");
    pass.set_viewport(viewport).expect("set_viewport failed");
    pass
}

/// Checks every pixel of a `width` by `height` image against `expected`.
fn assert_pixels(image: &[u8], width: u32, height: u32, expected: impl Fn(u32, u32) -> [u8; 4]) {
    assert_eq!(image.len(), (width * height * 4) as usize, "image size");
    for y in 0..height {
        for x in 0..width {
            let start = ((y * width + x) * 4) as usize;
            assert_eq!(image[start..start + 4], expected(x, y), "pixel ({x}, {y})");
        }
    }
}

/// Red inside `region` and blue everywhere else.
fn red_inside(region: ScreenRegion) -> impl Fn(u32, u32) -> [u8; 4] {
    move |x, y| {
        let inside =
            (region.min_x..region.max_x).contains(&x) && (region.min_y..region.max_y).contains(&y);
        if inside { RED } else { BLUE }
    }
}

// Story: A pass draws only inside its viewport and keeps the clear color outside it.
#[test]
fn draws_inside_the_viewport_only() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let pass = fill_pass([4, 2, 8, 6], [1.0, 0.0, 0.0, 1.0]);

        renderer.render(&pass, &target).expect("render failed");

        let image = target.get_image().await.expect("read back");
        assert_pixels(
            &image,
            16,
            16,
            red_inside(ScreenRegion::new((4, 2), (8, 6)).expect("a region")),
        );
    });
}

// Story: A viewport that reaches past the target draws into the part that fits.
#[test]
fn clips_a_viewport_larger_than_the_target() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");

        let past = fill_pass([10, 6, 100, 100], [1.0, 0.0, 0.0, 1.0]);
        renderer.render(&past, &target).expect("render failed");
        let image = target.get_image().await.expect("read back");
        assert_pixels(
            &image,
            16,
            16,
            red_inside(ScreenRegion::new((10, 6), (6, 10)).expect("a region")),
        );

        let whole = fill_pass([64, 64], [1.0, 0.0, 0.0, 1.0]);
        renderer.render(&whole, &target).expect("render failed");
        let image = target.get_image().await.expect("read back");
        assert_pixels(&image, 16, 16, |_, _| RED);
    });
}

// Story: A viewport that reaches past the target is cropped, not squeezed into the part that fits.
#[test]
fn crops_a_viewport_that_reaches_past_the_target() {
    const HALVES: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc_x: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    let p = array<vec2<f32>, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return VertexOutput(vec4<f32>(p[i], 0.0, 1.0), p[i].x);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.ndc_x < 0.0 { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
"#;
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(HALVES).expect("halves shader failed");
        let pass = Pass::from_shader("halves", &shader);
        pass.set_clear_color([0.0, 0.0, 1.0, 1.0])
            .expect("set_clear_color failed");
        // The viewport spans x 8..24, so its red left half covers 8..16 and its green half falls off.
        pass.set_viewport([8, 0, 16, 8])
            .expect("set_viewport failed");

        renderer.render(&pass, &target).expect("render failed");

        let image = target.get_image().await.expect("read back");
        assert_pixels(&image, 16, 8, |x, _| if x < 8 { BLUE } else { RED });
    });
}

// Story: A viewport with a side past the device's 2D texture limit is refused before
// the frame, where it shrank to the limit and squeezed the image to another zoom.
#[test]
fn refuses_a_viewport_past_the_device_limit() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 8])
            .await
            .expect("create_texture_target failed");
        // No device takes a side of 2^24, while a side of 4096 fits every one.
        let pass = fill_pass([0, 0, 1 << 24, 8], [1.0, 0.0, 0.0, 1.0]);

        let result = renderer.render(&pass, &target);
        assert!(
            matches!(
                &result,
                Err(fragmentcolor::RendererError::PassError(PassError::ViewportLimit { size, .. }))
                    if *size == [1 << 24, 8]
            ),
            "{result:?}"
        );

        pass.set_viewport([0, 0, 4096, 8])
            .expect("set_viewport failed");
        renderer
            .render(&pass, &target)
            .expect("a viewport within the limit");
    });
}

// Story: A viewport outside the target draws nothing, and the pass still clears.
#[test]
fn skips_draws_for_a_viewport_outside_the_target() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let pass = fill_pass([20, 0, 8, 8], [1.0, 0.0, 0.0, 1.0]);

        renderer.render(&pass, &target).expect("render failed");

        let image = target.get_image().await.expect("read back");
        assert_pixels(&image, 16, 16, |_, _| BLUE);
    });
}

// Story: A pass with its own target clips the viewport to that target's size.
#[test]
fn clips_to_the_pass_target() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let side = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let probe = renderer
            .create_texture_target([32, 32])
            .await
            .expect("create_texture_target failed");
        let pass = fill_pass([4, 0, 24, 24], [1.0, 0.0, 0.0, 1.0]);
        pass.set_target(&side).expect("set_target failed");

        renderer.render(&pass, &probe).expect("render failed");

        let image = side.get_image().await.expect("read back");
        assert_pixels(
            &image,
            8,
            8,
            red_inside(ScreenRegion::new((4, 0), (4, 8)).expect("a region")),
        );
    });
}

// Story: A second pass that loads the first one's output fills the other half of the target.
#[test]
fn splits_the_target_between_two_passes() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 8])
            .await
            .expect("create_texture_target failed");
        let left = fill_pass([0, 0, 8, 8], [1.0, 0.0, 0.0, 1.0]);
        let right = fill_pass([8, 0, 8, 8], [0.0, 1.0, 0.0, 1.0]);
        right.load_previous();

        renderer
            .render(&vec![&left, &right], &target)
            .expect("render failed");

        let image = target.get_image().await.expect("read back");
        assert_pixels(&image, 16, 8, |x, _| if x < 8 { RED } else { GREEN });
    });
}
