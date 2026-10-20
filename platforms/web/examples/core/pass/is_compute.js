import { Shader, Pass } from "fragmentcolor";

const shader = new Shader(`

@compute @workgroup_size(1)
fn cs_main() { }

`);
const pass = Pass.fromShader("p", shader);

// Call the method
const is_compute = pass.isCompute();
