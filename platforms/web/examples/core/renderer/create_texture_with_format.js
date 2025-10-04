import { Renderer, TextureFormat } from "fragmentcolor";
const renderer = new Renderer();
const image = "/healthcheck/public/favicon.png";
const tex = await renderer.createTextureWithFormat(image, TextureFormat.Rgba);