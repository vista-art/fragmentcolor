
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes
const pixels = [255,255,255,255];
const tex = await renderer.createTextureWithSize(pixels, [1, 1]);
const a = tex.aspect();
