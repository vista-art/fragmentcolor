import { Pass, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const depth = await renderer.createDepthTexture([800, 600]);
const pass = new Pass("scene");
pass.setDepthTarget(depth);

// The window grew. The pass keeps the same depth texture at the new size.
depth.resize([1024, 768]);
