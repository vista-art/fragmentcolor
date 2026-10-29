#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Material, Renderer, Shader, Target, TextureTarget};

/// A fullscreen triangle that draws texel (0, 0) of `base_color_map`.
const MAPPED: &str = r#"
@group(0) @binding(0) var base_color_map: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return textureLoad(base_color_map, vec2<i32>(0), 0);
}
"#;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];

/// A Material whose base color map is a 2x2 image of `color`, given as raw pixels.
fn build_material(color: [u8; 4]) -> Material {
    let shader = Shader::new(MAPPED).expect("a shader");
    Material::custom(shader).base_color_texture((color.repeat(4), [2, 2]))
}

/// A Renderer and an 8x8 target it made.
async fn open() -> (Renderer, TextureTarget) {
    let renderer = Renderer::new();
    let target = renderer
        .create_texture_target([8, 8])
        .await
        .expect("a target");
    (renderer, target)
}

/// The first pixel of `target`.
async fn read_pixel(target: &TextureTarget) -> [u8; 4] {
    let image = target.get_image().await.expect("read back");
    [image[0], image[1], image[2], image[3]]
}

// Story: one Material draws in two Renderers, each from a copy of its own.
#[test]
fn one_material_draws_in_two_renderers() {
    pollster::block_on(async move {
        let material = build_material(RED);
        let (first, first_target) = open().await;
        let (second, second_target) = open().await;

        first
            .render(material.shader(), &first_target)
            .expect("the first draws");
        second
            .render(material.shader(), &second_target)
            .expect("the second draws");

        assert_eq!(read_pixel(&first_target).await, RED);
        assert_eq!(read_pixel(&second_target).await, RED);
    });
}

// Story: a load in a second Renderer uploads its own copy of a map set again,
// and the first Renderer still draws, with the new map.
#[test]
fn loading_in_a_second_renderer_keeps_the_first() {
    pollster::block_on(async move {
        let material = build_material(RED);
        let (first, first_target) = open().await;
        let (second, second_target) = open().await;
        first
            .render(material.shader(), &first_target)
            .expect("the first draws");

        material.base_color_texture((GREEN.repeat(4), [2, 2]));
        second.load(material.shader()).await.expect("the load");
        first
            .render(material.shader(), &first_target)
            .expect("the first still draws");
        second
            .render(material.shader(), &second_target)
            .expect("the second draws");

        assert_eq!(read_pixel(&first_target).await, GREEN);
        assert_eq!(read_pixel(&second_target).await, GREEN);
    });
}

// Story: a map set again reaches every Renderer that draws it.
#[test]
fn a_later_set_wins_in_every_renderer() {
    pollster::block_on(async move {
        let material = build_material(RED);
        let (first, first_target) = open().await;
        let (second, second_target) = open().await;
        for (renderer, target) in [(&first, &first_target), (&second, &second_target)] {
            renderer.render(material.shader(), target).expect("draws");
            assert_eq!(read_pixel(target).await, RED);
        }

        material.base_color_texture((GREEN.repeat(4), [2, 2]));

        for (renderer, target) in [(&first, &first_target), (&second, &second_target)] {
            renderer.render(material.shader(), target).expect("draws");
            assert_eq!(read_pixel(target).await, GREEN);
        }
    });
}

// Story: a new Renderer draws a map after the Renderer that drew it first is gone,
// as an app does when it builds a new Renderer for a new surface.
#[test]
fn a_new_renderer_draws_after_the_first_drops() {
    pollster::block_on(async move {
        let material = build_material(RED);
        let (first, first_target) = open().await;
        first
            .render(material.shader(), &first_target)
            .expect("the first draws");
        drop((first, first_target));

        let (second, second_target) = open().await;
        second
            .render(material.shader(), &second_target)
            .expect("the new Renderer draws");

        assert_eq!(read_pixel(&second_target).await, RED);
    });
}
