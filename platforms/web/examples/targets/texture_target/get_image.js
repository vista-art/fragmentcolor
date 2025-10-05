import { Renderer, Shader } from "fragmentcolor";


const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);
renderer.render(new Shader(""), target);

const image = target.getImage();
