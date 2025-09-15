const bytes = std.fs.read("./examples/assets/image.png").unwrap();
const renderer = new Renderer();
const tex = futures.executor.blockOn(renderer.createTextureFromBytes(bytes)).unwrap();
shader.set("tex", tex).unwrap();