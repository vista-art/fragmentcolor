import { Renderer, Shader } from "fragmentcolor";

const r = new Renderer();
const target = await r.createTextureTarget([8, 8]);
const shader = Shader.default();
r.render(shader, target);
r.waitIdle();
const _bytes = target.getImage();