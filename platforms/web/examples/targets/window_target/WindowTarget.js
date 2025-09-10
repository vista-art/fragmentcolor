
import { Renderer, Shader, Target } from "fragmentcolor";

// Use your platform's windowing system to create a window.;
// We officially support Winit. Check the examples folder for details.;
const canvas = (()=>{const c=document.createElement('canvas');c.width=800;c.height=600;return c;})();

const renderer = new Renderer();
const target = await renderer.createTarget(canvas);

renderer.render(Shader.default(), target);
