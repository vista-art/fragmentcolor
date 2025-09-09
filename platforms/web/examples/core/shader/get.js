import { Shader } from "fragmentcolor";

const shader = Shader.default();
const _ = shader.set("resolution", [800.0, 600.0]);
const _res = shader.get("resolution");