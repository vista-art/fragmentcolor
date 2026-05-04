import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = Array(4 * 4 * 4).fill(200);
const chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [4, 4],
));
const _ = chain.format();