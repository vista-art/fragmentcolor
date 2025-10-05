
import { Renderer, Pass, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = Shader.default();
const pass = new Pass("blend with previous");
pass.addShader(shader);
pass.loadPrevious();

renderer.render(pass, target);
