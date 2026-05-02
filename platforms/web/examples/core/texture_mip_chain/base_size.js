import { TextureFormat, TextureMipChain } from "fragmentcolor";

const pixels = Array(16 * 16 * 4).fill(0);
const chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [16, 16],
));
const (width, height) = chain.baseSize();
const _ = (width, height);