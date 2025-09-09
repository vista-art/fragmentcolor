import { Pass, Shader } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("p");
pass.addShader(shader);