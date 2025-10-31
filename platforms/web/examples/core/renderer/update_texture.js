import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const id = *renderer;
    .createStorageTexture([640, 480], TextureFormat.Rgba, null);
await ;
    .id();

const width = 640u32;
const height = 480u32;
const pixel = 4u32;
const stride = width * pixel;
const align = wgpu.COPYBYTESPERROWALIGNMENT as u32;
const bpr = ((stride + align - 1) / align) * align;
const required = (bpr * (height - 1) + stride) as usize;
const frame = [0u8; required];

renderer.updateTexture(id, frame);