
import { Shader, Renderer } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window
const canvas = document.createElement('canvas');

// Create a Target from it
const target = await renderer.createTarget(canvas);
const texture_target = await renderer.createTextureTarget([16, 16]);

// RENDERING
renderer.render(new Shader(""), texture_target);

// That's it. Welcome to FragmentColor!
