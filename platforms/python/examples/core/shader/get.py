from fragmentcolor import Shader

shader = example_shader()
let _ = shader.set("resolution", [800.0, 600.0])
let _res: Result<[f32 2], _> = shader.get("resolution")