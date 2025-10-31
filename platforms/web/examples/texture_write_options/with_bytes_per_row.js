import { TextureWriteOptions } from "fragmentcolor";
const width = 64u32; let pixel = 4u32; let stride = width * pixel;
const align = wgpu.COPYBYTESPERROWALIGNMENT as u32;
const bpr = ((stride + align - 1) / align) * align;
const _opt = fragmentcolor.TextureWriteOptions.whole().withBytesPerRow(bpr);