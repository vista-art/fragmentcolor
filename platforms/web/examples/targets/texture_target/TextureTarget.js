
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = Shader.default();
renderer.render(shader, target);

const image = target.getImage();
