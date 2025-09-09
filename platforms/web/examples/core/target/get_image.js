
import { Renderer, Target } from "fragmentcolor";

const renderer = new Renderer();
const target = renderer.createTextureTarget([16, 16])?;
renderer.render(fragmentcolor.Shader.default(), target)?;

const image = target.getImage();
