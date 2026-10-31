import { Mipmap, TextureFormat } from "fragmentcolor";
const pixels = new Uint8Array(16 * 16 * 4);

const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16]);
const size = chain.size();
const width = size.width;
const height = size.height;