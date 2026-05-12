import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const texture = await renderer.createTexture([ 255, 200, 120, 255, 255,  240, 180, 255, 230,  180, 100, 255, 255,  220, 150, 255, ][..]);
const mat = Material.pbr().baseColorTexture(texture);