import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const satin = await Material.pbr(renderer).roughness(0.35);