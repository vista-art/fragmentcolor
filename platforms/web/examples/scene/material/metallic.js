import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const chrome = Material.pbr()?.metallic(1.0).roughness(0.05);