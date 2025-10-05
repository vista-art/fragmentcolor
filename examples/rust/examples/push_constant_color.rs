use fragmentcolor::{Renderer, Shader, Target};

// Minimal example: push constants controlling fragment color.
// Renders offscreen into a 512x512 texture target.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        let renderer = Renderer::new();

        // WGSL with a push-constant color used by the fragment shader
        let wgsl = r#"
struct PC { color: vec4<f32> };
var<push_constant> pc: PC;

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}

@fragment fn fs_main() -> @location(0) vec4<f32> {
  return pc.color;
}
        "#;

        let shader = Shader::new(wgsl)?;
        // Set the push-constant color to red with full opacity
        shader.set("pc.color", [1.0, 0.0, 0.0, 1.0])?;

        // Render offscreen
        let target = renderer.create_texture_target([512u32, 512u32]).await?;
        renderer.render(&shader, &target)?;

        assert_eq!(target.size().width, 512);
        assert_eq!(target.size().height, 512);
        let pixels = target.get_image();
        assert_eq!(pixels.len(), 512 * 512 * 4);

        Ok(())
    })
}
