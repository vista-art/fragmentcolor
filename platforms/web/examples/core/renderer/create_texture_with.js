import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
const pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
];
const tex = await renderer.createTextureWith(pixels, [2, 2]);