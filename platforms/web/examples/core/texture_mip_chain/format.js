import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = new Uint8Array(4 * 4 * 4);
pixels.fill(200);
const chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [4, 4]);
const _ = chain.format();