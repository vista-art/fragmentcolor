from fragmentcolor import Renderer, TextureFormat, TextureWriteOptions
renderer = Renderer()
id = *renderer
    .create_storage_texture([640, 480], TextureFormat.Rgba, None)
    
    .id()

width = 640u32
height = 480u32
pixel = 4u32
stride = width * pixel
align = wgpu.COPY_BYTES_PER_ROW_ALIGNMENT as u32
bpr = ((stride + align - 1) / align) * align
required = (bpr * (height - 1) + stride) as usize
frame = [0u8; required]
opt = TextureWriteOptions.whole().with_bytes_per_row(bpr)

renderer.update_texture_with(id, frame, opt)
``