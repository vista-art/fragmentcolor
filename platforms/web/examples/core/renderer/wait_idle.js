import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget({ width: 8, height: 8 });
const shader = new Shader("void main() { fragColor = vec4(1.0); }");
renderer.render(shader, target);
renderer.waitIdle();
const bytes = await target.getImage();