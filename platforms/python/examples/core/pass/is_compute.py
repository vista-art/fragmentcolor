from fragmentcolor import Shader, Pass, Mesh, Vertex

wgsl = r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#
shader = Shader(wgsl)
rpass = Pass("p"); rpass.add_shader(shader)

if rpass.is_compute() {
    println!("This is a compute rpass.")
}
