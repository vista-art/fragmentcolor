use fragmentcolor::Target;
use fragmentcolor::mesh::Mesh;
use fragmentcolor::mesh::primitives::Quad;
use fragmentcolor::{Pass, Renderer, Shader};

fn main() {
    pollster::block_on(async move {
        // Renderer and headless target
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([256u32, 256u32])
            .await
            .expect("texture target");

        // WGSL: sample texture using pos+uv
        let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.uv = uv;
  return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
  return textureSample(tex, samp, v.uv);
}
        "#;

        let shader = Shader::new(wgsl).expect("shader");
        let pass = Pass::from_shader("two_quads", &shader);

        // Create a tiny 2x2 RGBA texture: red, green, blue, white
        #[rustfmt::skip]
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ];
        let tex = renderer
            .create_texture_with_size(&pixels, [2u32, 2u32])
            .await
            .expect("texture");
        shader.set("tex", &tex).expect("set tex");

        // Build two quads with different positions (clip-space)
        let left: Mesh = Quad::new([-0.9, -0.5], [-0.1, 0.5]).into();
        let right: Mesh = Quad::new([0.1, -0.5], [0.9, 0.5]).into();

        // Attach meshes to the shader in this pass
        pass.add_mesh_to_shader(&left, &shader).expect("left ok");
        pass.add_mesh_to_shader(&right, &shader).expect("right ok");

        // Render
        renderer.render(&pass, &target).expect("render ok");

        // Optionally: read back image; here we just assert size
        let image = target.get_image();
        assert_eq!(image.len(), 256 * 256 * 4);
    });
}
