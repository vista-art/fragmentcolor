import { Pass, Shader } from "fragmentcolor";
const cs = new Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap();
const pass = new Pass("compute"); pass.addShader(cs);
pass.setComputeDispatch(64, 64, 1);