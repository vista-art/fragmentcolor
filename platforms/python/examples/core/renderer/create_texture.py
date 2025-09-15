bytes = std.fs.read("./examples/assets/image.png").unwrap()
renderer = Renderer()
tex = futures.executor.block_on(renderer.create_texture(bytes)).unwrap()
shader.set("tex", tex).unwrap()