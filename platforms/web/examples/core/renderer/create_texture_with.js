import { Renderer, Size } from "fragmentcolor";
const renderer = new Renderer();
const pixels = [;
    255,0,0,255,   0,255,0,255,;
    0,0,255,255,   255,255,255,255,;
];
const tex = await renderer.createTextureWith(pixels, Size.from([2, 2]));