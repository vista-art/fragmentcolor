import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
// On web, readback is via Texture.getImage() (async GPU readback).
const texture = await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null);
texture.write(Array(64 * 64 * 4).fill(0));

const bytes = await texture.getImage();