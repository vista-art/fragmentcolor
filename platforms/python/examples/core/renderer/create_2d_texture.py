r = Renderer()
size = Size.from((256, 256))
pixels = [255u8; (256*256*4) as usize]
tex = futures.executor.block_on(
    r.create_2d_texture(pixels, Some(size), Some(wgpu.TextureFormat.Rgba8UnormSrgb))
).unwrap()