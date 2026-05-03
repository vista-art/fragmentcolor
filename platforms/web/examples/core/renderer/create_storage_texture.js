import { Renderer, TextureFormat } from "fragmentcolor";

const r = new Renderer();
// Empty storage texture: size, format, data=null
const tex = await r.createStorageTexture([64, 64], TextureFormat.Rgba, null);

// Pre-seeded with bytes: pass raw data as third argument.
const pixels = new Uint8Array(64 * 64 * 4);
const tex2 = await r.createStorageTexture([64, 64], TextureFormat.Rgba, pixels);
