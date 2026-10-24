import { Mipmap, TextureFormat } from "fragmentcolor";
const pixels = new Uint8Array(8 * 8 * 4);

const chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8]);
const level_zero_bytes = chain.levels()[0];