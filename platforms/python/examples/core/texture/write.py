from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
texture = renderer.create_storage_texture([1280, 720], TextureFormat.Rgba, None)

width = 1280u32
height = 720u32
pixel_size = 4u32; # RGBA8
stride = width * pixel_size
align = wgpu.COPY_BYTES_PER_ROW_ALIGNMENT as u32
bpr = ((stride + align - 1) / align) * align; # align to 256

required = (bpr * (height - 1) + stride) as usize
frame_bytes = [0u8; required]

texture.write(frame_bytes)