const renderer = new Renderer();
const tex = futures.executor.blockOn(renderer.createTextureFromFile("./examples/assets/image.png")).unwrap();
shader.set("tex", tex).unwrap();