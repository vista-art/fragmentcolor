
import { Renderer } from "fragmentcolor";

const renderer = new Renderer();

// Use your platform's windowing system to create a window.
const canvas = document.createElement('canvas');

const target = await renderer.createTarget(canvas);
