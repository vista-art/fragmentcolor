import { Renderer, Shader } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);
const shader = Shader.default();
renderer.render(shader, target);

// The same frame, drawn where it can be read back.
const offscreen = await renderer.createTextureTarget([800, 600]);
renderer.render(shader, offscreen);
const image = await offscreen.getImage();