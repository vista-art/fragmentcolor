import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const texture = await renderer.createStorageTexture(([64, 64], TextureFormat.Rgba));
texture.write(Array(64 * 64 * 4).fill(0));

const bytes = await renderer.readTextureAsync(texture.id());