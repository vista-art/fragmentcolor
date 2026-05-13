import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const chrome = await Material.pbr(renderer).metallic(1.0).roughness(0.05);