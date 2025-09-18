const bytes = std.fs.read("./examples/assets/image.png").unwrap();
const renderer = new Renderer();
const tex = futures.executor.blockOn(renderer.createTexture(bytes)).unwrap();
// use in a shader uniform;
// shader.set("tex", tex);