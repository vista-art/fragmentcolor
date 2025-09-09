import { Shader } from "fragmentcolor";

const shader = exampleShader();
let _ = shader.set("resolution", [800.0, 600.0]);
let _res: Result<[f32; 2], _> = shader.get("resolution");