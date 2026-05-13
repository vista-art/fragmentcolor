import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const crevices = await Material.pbr(renderer).occlusionStrength(0.8);