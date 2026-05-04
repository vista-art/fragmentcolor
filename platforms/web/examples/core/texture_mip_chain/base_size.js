import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = new Uint8Array(16 * 16 * 4);
const chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16]);
const sz = chain.baseSize();
const width = sz.width;
const height = sz.height;