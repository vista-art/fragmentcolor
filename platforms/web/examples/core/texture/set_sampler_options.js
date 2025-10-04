import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes
const pixels = [255,255,255,255];

const texture = await renderer.createTextureWithSize(pixels, [1,1]);
const opts = {repeat_x: true, repeat_y: true, smooth: true, compare: null};
texture.setSamplerOptions(opts);

