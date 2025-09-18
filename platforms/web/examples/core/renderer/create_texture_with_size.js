const renderer = new Renderer();
const size = Size.from((2, 2));
const pixels = [;
    255,0,0,255,   0,255,0,255,;
    0,0,255,255,   255,255,255,255,;
];
const tex = renderer;
    .createTextureWith(pixels, size);
await ;