import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
// Load encoded image bytes (PNG/JPEG) or use a file path
const image = "/healthcheck/public/favicon.png";
const tex = await renderer.createTexture(image);