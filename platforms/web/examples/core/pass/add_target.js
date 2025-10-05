import { Renderer, Pass, TextureFormat } from "fragmentcolor";

const r = new Renderer();
const tex_target = await r.createTextureTarget([512, 512]);

const p = new Pass("shadow");
p.addTarget(tex_target);
