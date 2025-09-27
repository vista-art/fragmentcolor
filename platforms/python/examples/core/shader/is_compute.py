from fragmentcolor import Shader, Pass, Mesh, Vertex

wgsl = r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#
shader = Shader(wgsl)

if shader.is_compute() {
    println!("This is a compute shader.")
}
