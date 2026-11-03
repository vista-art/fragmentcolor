import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([16, 16]);
const shader = Shader.default();

// A GPU error in this frame returns as a DeviceError.
renderer.render(shader, target);