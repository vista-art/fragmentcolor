import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = Array(8 * 8 * 4).fill(0);
const chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [8, 8],
));
const level_zero_bytes = chain.levels()[0];
const _ = level_zero_bytes;