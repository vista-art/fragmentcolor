
import { Shader } from "fragmentcolor";

const shader = Shader.default();

// The default shader includes a "resolution" uniform
shader.set("resolution", [800.0, 600.0]);
const res = shader.get("resolution");
