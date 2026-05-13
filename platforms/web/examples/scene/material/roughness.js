import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const satin = Material.pbr()?.roughness(0.35);