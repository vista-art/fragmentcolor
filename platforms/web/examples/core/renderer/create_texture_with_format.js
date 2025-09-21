import { Renderer, Size, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const bytes = std.fs.read("logo.png");
const tex = await renderer.createTextureWithFormat(bytes, TextureFormat.Rgba);