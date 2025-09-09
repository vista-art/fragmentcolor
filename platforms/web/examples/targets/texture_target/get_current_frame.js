import { Renderer } from "fragmentcolor";


const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);
const frame = target.getCurrentFrame()?;
const _format = frame.format();
