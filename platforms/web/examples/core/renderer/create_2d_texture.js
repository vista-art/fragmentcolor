const r = new Renderer();
const size = Size.from((256, 256));
const pixels = [255u8; (256*256*4) as usize];
const tex = futures.executor.blockOn(;
    r.create2dTexture(pixels, Some(size), Some(wgpu.TextureFormat.Rgba8UnormSrgb));
).unwrap();