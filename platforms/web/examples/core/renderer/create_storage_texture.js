
import { Renderer, TextureFormat } from "fragmentcolor";

const r = new Renderer();
const tex = await r.createStorageTexture([64, 64], TextureFormat.Rgba, null);
