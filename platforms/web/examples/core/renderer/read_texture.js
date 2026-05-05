import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture([64, 64], TextureFormat.Rgba);
texture.write(new Uint8Array(64 * 64 * 4));

const bytes = await renderer.readTexture(texture.id());