import { TextureFormat, Mipmap } from "fragmentcolor";

const pixels = new Uint8Array(16 * 16 * 4);
const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16]);
const sz = chain.size();
const width = sz.width;
const height = sz.height;