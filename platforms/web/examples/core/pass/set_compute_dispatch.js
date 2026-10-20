import { Pass, Shader } from "fragmentcolor";
const cs = new Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}");
const pass = Pass.fromShader("compute", cs);
pass.setComputeDispatch(64, 64, 1);