import { Mipmap, Renderer, TextureFormat } from "fragmentcolor";
const encoded_png_bytes = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
const raw_rgba = new Uint8Array(8 * 8 * 4).fill(200);

// Encoded path: bytes plus the format you want the chain to live in.
// The dimensions come from the decoded image.
const chain = Mipmap.build(encoded_png_bytes, TextureFormat.Rgba8UnormSrgb);

// Raw path: include the size so build skips decoding.
const chain_raw = Mipmap.build(raw_rgba, TextureFormat.Rgba8UnormSrgb, [8, 8]);

// Either chain uploads the same way.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);