
import { Renderer, Shader } from "fragmentcolor";
const renderer = new Renderer();
const shader = Shader.default();

const bytes = std.fs.read("./examples/assets/image.png").unwrap();
const texture = await renderer.createTexture(bytes);

shader.set("texture", texture).unwrap();
