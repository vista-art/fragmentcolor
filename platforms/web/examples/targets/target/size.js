
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 32]);
const size = target.size();
const width = size.width;
const height = size.height;
const depth = size.depth;
