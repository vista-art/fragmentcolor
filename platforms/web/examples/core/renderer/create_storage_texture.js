import { Renderer, TextureFormat } from "fragmentcolor";

const r = new Renderer();

// Empty storage texture.
const tex = await r.createStorageTexture([64, 64], TextureFormat.Rgba);

// Pre-seeded with bytes.
const pixels = Array(64 * 64 * 4).fill(0);
const tex2 = await r.createStorageTexture([64, 64], TextureFormat.Rgba, pixels);