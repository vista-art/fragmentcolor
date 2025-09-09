import { Pass, Shader } from "fragmentcolor";

const shader = exampleShader();
const pass = new Pass("p");
pass.add_shader(&shader);