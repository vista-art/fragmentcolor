import { Renderer } from "fragmentcolor";
const renderer = new Renderer();
// Load encoded image bytes (PNG/JPEG) or use a file path;
const bytes = std.fs.read("./examples/assets/image.png");
const tex = await renderer.createTexture(bytes);