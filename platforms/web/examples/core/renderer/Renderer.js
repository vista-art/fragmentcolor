
import { Shader, Renderer } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();

// The canvas comes from your platform's windowing system
const target = await renderer.createTarget(canvas);
const texture_target = await renderer.createTextureTarget([16, 16]);

// RENDERING
renderer.render(Shader.default(), texture_target);

// That's it. Welcome to FragmentColor!
