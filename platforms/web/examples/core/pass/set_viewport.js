
import { Renderer, Pass, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = Shader.default();
const pass = new Pass("clipped");
pass.addShader(shader);

pass.setViewport({ x: 0, y: 0, width: 32, height: 32 });

renderer.render(pass, target);
