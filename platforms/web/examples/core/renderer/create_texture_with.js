import { Renderer, Size } from "fragmentcolor";
const renderer = new Renderer();
const size = Size.from((2, 2));
const pixels = [;
    255,0,0,255,   0,255,0,255,;
    0,0,255,255,   255,255,255,255,;
];
const tex = await renderer.createTextureWith(pixels, size);