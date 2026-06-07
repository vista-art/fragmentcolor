import { Renderer, TextureFormat, Mipmap } from "fragmentcolor";

// Encoded path - build from PNG / JPEG bytes (size is inferred).
// Use a fixture you have on hand. The favicon below is served by the
// healthcheck server so the example runs end-to-end without the test
// having to bring its own bytes.
const pngResp = await fetch("/healthcheck/public/favicon.png");
const pngBytes = new Uint8Array(await pngResp.arrayBuffer());
const chain = Mipmap.build(pngBytes, TextureFormat.Rgba8UnormSrgb);

// Raw pixel path: include the size so build skips decoding.
const rawRgba = new Uint8Array(8 * 8 * 4);
rawRgba.fill(200);
const chainRaw = Mipmap.build(rawRgba, TextureFormat.Rgba8UnormSrgb, [8, 8]);

// Upload the chain through the regular createTexture entry point.
const renderer = new Renderer();
const texture = await renderer.createTexture(chain);
const _ = chainRaw.count();
const __ = texture.size();