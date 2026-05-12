import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const normal_map = await renderer.createTexture([ 128, 128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, ][..]);
const mat = Material.pbr().normalTexture(normal_map).normalScale(1.2);