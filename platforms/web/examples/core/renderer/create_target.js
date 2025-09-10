
import { Renderer, Target } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.;
// We officially support Winit. Check the examples folder for details.;
const canvas = (()=>{const c=document.createElement('canvas');c.width=800;c.height=600;return c;})();

const target = renderer.createTarget(canvas);
