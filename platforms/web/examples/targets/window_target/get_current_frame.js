import { Renderer, Target } from "fragmentcolor";


const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);
const frame = target.getCurrentFrame()?; // Acquire a frame;
const _format = frame.format();
