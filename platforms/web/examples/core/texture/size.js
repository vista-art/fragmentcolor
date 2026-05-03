import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
const pixels = new Uint8Array([255,255,255,255]);
const tex = await renderer.createTexture(pixels, { size: [1, 1] });
const sz = tex.size();