# Assuming `renderer` and `shader` exist
bytes = std.fs.read("./examples/assets/image.png").unwrap()
tex = futures.executor.block_on(renderer.create_texture_from_bytes(bytes)).unwrap()
shader.set("tex", tex).unwrap()