import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const normal_map_bytes = [ 128, 128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, ];
const normal_map = await renderer.createTexture(normal_map_bytes, [2, 2]);
const mat = Material.pbr().normalTexture(normal_map).normalScale(1.2);