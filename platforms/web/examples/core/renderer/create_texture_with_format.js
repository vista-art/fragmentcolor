import { Renderer, Size, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const image = std.fs.read("logo.png");
const tex = await renderer.createTextureWithFormat(image, TextureFormat.Rgba);