import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const ao = await renderer.createTexture([ 220, 0, 0, 255, 180,   0, 0, 255, 200,   0, 0, 255, 160,   0, 0, 255, ][..]);
const mat = Material.pbr().occlusionTexture(ao);