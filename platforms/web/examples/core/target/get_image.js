
import { Shader, Renderer, Target } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);
renderer.render(Shader.default(), target);

const image = target.getImage();
