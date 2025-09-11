
import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.;
// We officially support Winit. Check the examples folder for details.;
const canvas = document.createElement('canvas');

// You can create multiple targets from the same Renderer.;
const target = await renderer.createTarget(canvas);
const target2 = await renderer.createTarget(canvas);

// To animate, render again in your event loop...;
renderer.render(Shader.default(), target);
renderer.render(Shader.default(), target2);
