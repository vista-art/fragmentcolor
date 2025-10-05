
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

target.resize([128, 32]);
