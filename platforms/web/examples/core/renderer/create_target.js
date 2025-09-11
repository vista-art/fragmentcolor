
import { Renderer, Target } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.;
// We officially support Winit. Check the examples folder for details.;
const canvas = document.createElement('canvas');

const target = renderer.createTarget(canvas);
