import { Renderer, TextureFormat } from "fragmentcolor";

const r = new Renderer();
const tex = await r.createRenderTexture([256, 256], TextureFormat.Rgba8Unorm);