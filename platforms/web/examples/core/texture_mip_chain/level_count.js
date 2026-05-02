import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = Array(8 * 8 * 4).fill(0);
const chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [8, 8],
));
const count = chain.levelCount();
const _ = count;