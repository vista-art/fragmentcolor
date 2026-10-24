import { Mipmap, TextureFormat } from "fragmentcolor";
const pixels = new Uint8Array(4 * 4 * 4).fill(200);

const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [4, 4]);
const format = chain.format();