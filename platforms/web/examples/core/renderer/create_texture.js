import { Renderer } from "fragmentcolor";
const image = "/healthcheck/public/favicon.png";
const renderer = new Renderer();
const tex = await renderer.createTexture(image);
// use in a shader uniform
// shader.set("tex", tex);