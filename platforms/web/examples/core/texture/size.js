const renderer = new Renderer();
// 1x1 RGBA (white) raw pixel bytes;
const pixels = [255,255,255,255];
const tex = await renderer.createTextureWithSize(pixels, Size.from((1,1)));
const sz = tex.size();