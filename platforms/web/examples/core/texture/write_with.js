import { Renderer, TextureFormat, TextureWriteOptions } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture([640, 480], TextureFormat.Rgba, null);

// Upload a 320x240 region starting at (x=100, y=50)
const w = 320u32;
const h = 240u32;
const pixel = 4u32;
const stride = w * pixel;
const align = wgpu.COPYBYTESPERROWALIGNMENT as u32;
const bpr = ((stride + align - 1) / align) * align;
const required = (bpr * (h - 1) + stride) as usize;
const region_bytes = [0u8; required];
const opt = TextureWriteOptions {
    origin_x: 100,
    origin_y: 50,
    origin_z: 0,
    size_width: w,
    size_height: h,
    size_depth: 1,
    bytes_per_row: Some(bpr),
    rows_per_image: Some(h),
};
texture.writeWith(region_bytes, opt);
``;