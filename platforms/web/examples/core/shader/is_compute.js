import { Shader } from "fragmentcolor";

const shader = new Shader(`
@compute @workgroup_size(1)
fn cs_main() { }

`);

// Call the method
const is_compute = shader.isCompute();
