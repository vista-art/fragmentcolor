import { Renderer, Size, SamplerOptions } from "fragmentcolor";
const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes;
const pixels = [255,255,255,255];
const tex = await renderer.createTextureWithSize(pixels, Size.from((1,1)));
const opts = SamplerOptions { repeat_x: true, repeat_y: true, smooth: true, compare: None };
tex.setSamplerOptions(opts);