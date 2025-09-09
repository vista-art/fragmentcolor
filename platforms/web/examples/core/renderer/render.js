
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([10, 10]);
const shader = Shader.default();

renderer.render(shader, target);
