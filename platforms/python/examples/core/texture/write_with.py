from fragmentcolor import Renderer, TextureFormat, TextureWriteOptions
renderer = Renderer()
texture = renderer.create_storage_texture([640, 480], TextureFormat.Rgba, None)

# Upload a 320x240 region starting at (x=100, y=50)
w = 320u32
h = 240u32
pixel = 4u32
stride = w * pixel
align = wgpu.COPY_BYTES_PER_ROW_ALIGNMENT as u32
bpr = ((stride + align - 1) / align) * align
required = (bpr * (h - 1) + stride) as usize
region_bytes = [0u8; required]
opt = TextureWriteOptions {
    origin_x: 100,
    origin_y: 50,
    origin_z: 0,
    size_width: w,
    size_height: h,
    size_depth: 1,
    bytes_per_row: Some(bpr),
    rows_per_image: Some(h),
}
texture.write_with(region_bytes, opt)
``