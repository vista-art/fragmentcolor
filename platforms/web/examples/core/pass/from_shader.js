import { Pass, Shader } from "fragmentcolor";

const shader = Shader.default();
const pass = Pass.fromShader("single", shader);