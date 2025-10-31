import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture([1280, 720], TextureFormat.Rgba, null);

const width = 1280u32;
const height = 720u32;
const pixel_size = 4u32; // RGBA8;
const stride = width * pixel_size;
const align = wgpu.COPYBYTESPERROWALIGNMENT as u32;
const bpr = ((stride + align - 1) / align) * align; // align to 256;

const required = (bpr * (height - 1) + stride) as usize;
const frame_bytes = [0u8; required];

texture.write(frame_bytes);