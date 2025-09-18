const r = new Renderer();
const size = fragmentcolor.Size { width: 16, height: 16, depth: Some(8) };
const voxels = [0u8; (16*16*8*4) as usize];
const tex = futures.executor.blockOn(;
    r.create3dTexture(voxels, Some(size), Some(wgpu.TextureFormat.Rgba8Unorm));
).unwrap();