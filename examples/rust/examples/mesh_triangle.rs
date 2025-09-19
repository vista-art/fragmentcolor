use fragmentcolor::mesh::{Mesh, Vertex};
use fragmentcolor::{Pass, Renderer, Shader};

fn main() {
    pollster::block_on(async move {
        // Create a headless texture target
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([512u32, 512u32])
            .await
            .expect("create texture target");

        // Minimal WGSL with vertex position only; draws a green triangle
        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
        "#;

        let shader = Shader::new(wgsl).expect("shader");
        let pass = Pass::from_shader("mesh", &shader);

        // Build a simple triangle mesh
        let mut mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::from([-0.5, -0.5, 0.0]),
            Vertex::from([0.5, -0.5, 0.0]),
            Vertex::from([0.0, 0.5, 0.0]),
        ]);

        pass.add_mesh(&mesh);
        renderer.render(&pass, &target).expect("render");

        // Optionally, read back image bytes: target.get_image()
    });
}
