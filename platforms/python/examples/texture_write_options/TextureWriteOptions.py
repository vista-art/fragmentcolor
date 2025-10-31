from fragmentcolor import TextureWriteOptions
width = 64u32; let height = 64u32
pixel = 4u32; let stride = width * pixel
align = wgpu.COPY_BYTES_PER_ROW_ALIGNMENT as u32
bpr = ((stride + align - 1) / align) * align
_opt = fragmentcolor.TextureWriteOptions.whole()
  .with_bytes_per_row(bpr)
  .with_rows_per_image(height)