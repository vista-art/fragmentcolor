import { Pass, Shader } from "fragmentcolor";

const shader = exampleShader();
let pass = Pass::from_shader("single", &shader);