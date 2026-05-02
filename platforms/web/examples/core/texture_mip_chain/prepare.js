import { Renderer, TextureFormat, TextureMipChain } from "fragmentcolor";

// Encoded path â single tuple, no extra method.
const chain = TextureMipChain.prepare((encoded_png_bytes, TextureFormat.Rgba8UnormSrgb));

// Raw pixel path â same method, just include the size in the tuple.
const chain_raw = TextureMipChain.prepare((
    raw_rgba.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [8, 8],
));

// Hand the chain to the unified create_texture entry â same vocabulary.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);