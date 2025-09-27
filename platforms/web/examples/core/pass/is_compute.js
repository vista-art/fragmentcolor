import { Shader, Pass, Mesh, Vertex } from "fragmentcolor";

const wgsl = r#";
@compute @workgroup_size(1);
fn cs_main() { };
"#;
const shader = new Shader(wgsl).unwrap();
const pass = new Pass("p"); pass.addShader(shader);

if pass.isCompute() {;
    println!("This is a compute pass.");
};
