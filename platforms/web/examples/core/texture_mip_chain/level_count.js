import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = new Uint8Array(8 * 8 * 4);
const chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8]);
const count = chain.levelCount();
const _ = count;