
import { Renderer, Pass, Shader, Region } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = Shader.default();
const pass = new Pass("clipped");
pass.addShader(shader);

pass.setViewport([(0, 0), (32, 32)]);

renderer.render(pass, target);
