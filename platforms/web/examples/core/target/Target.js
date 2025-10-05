
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.
// We officially support Winit. Check the examples folder for details.
const canvas = document.createElement('canvas');

const target = await renderer.createTarget(canvas);

// To animate, render again in your event loop...
renderer.render(new Shader(""), target);
renderer.render(new Shader(""), target);
