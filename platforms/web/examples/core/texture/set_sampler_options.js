import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes
const pixels = new Uint8Array([255, 255, 255, 255]);
const texture = await renderer.createTexture(pixels, { size: [1, 1] });
const opts = { repeatX: true, repeatY: true, smooth: true, compare: null };
texture.setSamplerOptions(opts);