
import { Renderer, Pass, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = Shader.default();
const pass = new Pass("solid background");
pass.addShader(shader);

pass.setClearColor([0.1, 0.2, 0.3, 1.0]);

renderer.render(pass, target);
