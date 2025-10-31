import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null);
const id = *texture.id();