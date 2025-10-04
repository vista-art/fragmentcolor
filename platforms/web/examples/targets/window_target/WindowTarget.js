
import { Renderer, Shader } from "fragmentcolor";

// Use your platform's windowing system to create a window.
// We officially support Winit. Check the examples folder for details.
const canvas = document.createElement('canvas');

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);

renderer.render(new Shader(""), target);
