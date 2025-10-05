
import { Renderer, Shader } from "fragmentcolor";
const renderer = new Renderer();

// Create an offscreen texture target with a size of 64x64 pixels.
const target = await renderer.createTextureTarget([64, 64]);

renderer.render(new Shader(""), target);

// get the rendered image
const image = target.getImage();
