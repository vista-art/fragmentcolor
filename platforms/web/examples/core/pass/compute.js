import { Pass, Shader } from "fragmentcolor";

const cs = new Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}");
const pass = Pass.compute("compute");
pass.addShader(cs);