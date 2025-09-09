import { Pass, Shader } from "fragmentcolor";

const shader = Shader.default();
const pass = new Pass("single"); pass.addShader(shader);