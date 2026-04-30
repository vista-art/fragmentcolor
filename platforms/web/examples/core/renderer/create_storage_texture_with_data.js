import { Renderer, TextureFormat } from "fragmentcolor";

const r = new Renderer();
const seed = Array(8 * 8 * 4).fill(0);
const tex = await r.createStorageTextureWithData([8, 8], TextureFormat.Rgba, seed, null);
