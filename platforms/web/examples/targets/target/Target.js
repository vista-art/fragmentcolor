
import { Renderer, Shader } from "fragmentcolor";
const canvas = document.createElement("canvas");

const renderer = new Renderer();

// The canvas comes from your platform's windowing system.
const target = await renderer.createTarget(canvas);

// To animate, render again in your event loop...
renderer.render(Shader.default(), target);
renderer.render(Shader.default(), target);
