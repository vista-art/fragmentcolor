import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
const pixels = [255,255,255,255];
const tex = await renderer.createTextureWithSize(pixels, [1,1]);
const sz = tex.size();