renderer = Renderer()
tex = futures.executor.block_on(renderer.create_texture_from_file("./examples/assets/image.png")).unwrap()
shader.set("tex", tex).unwrap()