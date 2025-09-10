import { Renderer, Shader } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.;
// We officially support Winit. Check the examples folder for details.;
const canvas = (()=>{const c=document.createElement('canvas');c.width=800;c.height=600;return c;})();

// You can create multiple targets from the same Renderer.;
const target = await renderer.createTarget(canvas);
const target2 = await renderer.createTarget(canvas);

// To animate, render again in your event loop...;
renderer.render(Shader.default(), target);
renderer.render(Shader.default(), target2);
