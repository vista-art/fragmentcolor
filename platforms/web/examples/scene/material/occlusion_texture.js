import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const ao_bytes = [ 220, 0, 0, 255, 180,   0, 0, 255, 200,   0, 0, 255, 160,   0, 0, 255, ];
const ao = await renderer.createTexture(ao_bytes, [2, 2]);
const mat = Material.pbr().occlusionTexture(ao);