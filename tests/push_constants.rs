#![cfg(not(wasm))]

use fragmentcolor::{Renderer, Shader, Target};

// Story: Single push-constant root renders a solid color using native push constants (or fallback if needed)
#[test]
fn push_constant_single_root_renders_color() {
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target([8, 8])).expect("create target");

    let wgsl = r#"
struct PC { color: vec4<f32> };
var<push_constant> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return pc.color; }
"#;

    let shader = Shader::new(wgsl).expect("shader");
    // Solid green
    shader
        .set("pc.color", [0.0f32, 1.0, 0.0, 1.0])
        .expect("set pc.color");

    renderer.render(&shader, &target).expect("render");
    let img = target.get_image();
    assert!(img.len() >= 4);
    // Expect RGBA8 (0,255,0,255) on the first pixel
    assert_eq!([img[0], img[1], img[2], img[3]], [0u8, 255, 0, 255]);
}

// Story: Multiple push-constant roots trigger fallback and still render the expected color
#[test]
fn push_constant_multi_root_fallback_renders_color() {
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target([8, 8])).expect("create target");

    let wgsl = r#"
struct A { v: f32 };
struct B { color: vec4<f32> };
var<push_constant> a: A;
var<push_constant> b: B;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return b.color; }
"#;

    let shader = Shader::new(wgsl).expect("shader");
    shader.set("a.v", 1.0f32).expect("set a.v");
    // Blue
    shader
        .set("b.color", [0.0f32, 0.0, 1.0, 1.0])
        .expect("set b.color");

    renderer.render(&shader, &target).expect("render");
    let img = target.get_image();
    assert!(img.len() >= 4);
    assert_eq!([img[0], img[1], img[2], img[3]], [0u8, 0, 255, 255]);
}
