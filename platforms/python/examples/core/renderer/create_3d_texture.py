r = Renderer()
size = fragmentcolor.Size { width: 16, height: 16, depth: Some(8) }
voxels = [0u8; (16*16*8*4) as usize]
tex = futures.executor.block_on(
    r.create_3d_texture(voxels, Some(size), Some(wgpu.TextureFormat.Rgba8Unorm))
).unwrap()