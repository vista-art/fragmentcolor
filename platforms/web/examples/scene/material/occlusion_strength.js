import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const crevices = Material.pbr()?.occlusionStrength(0.8);