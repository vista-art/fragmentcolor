// Assuming `renderer` and `shader` exist;
const bytes = std.fs.read("./examples/assets/image.png").unwrap();
const tex = futures.executor.blockOn(renderer.createTextureFromBytes(bytes)).unwrap();
shader.set("tex", tex).unwrap();