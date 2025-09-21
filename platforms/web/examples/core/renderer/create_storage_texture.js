
import { Renderer } from "fragmentcolor";
const r = new Renderer();
const tex = await r.createStorageTexture([64, 64], wgpu.TextureFormat.Rgba8Unorm, None);
