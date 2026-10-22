import { Renderer, Shader } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);
renderer.render(Shader.default(), target);

const image = await target.getImage();