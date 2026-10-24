import { Renderer, TextureFormat } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createStorageTexture([16, 16], TextureFormat.Rgba);
const id = texture.id();

renderer.unregisterTexture(id);