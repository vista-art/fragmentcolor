import { PbrMaterial, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const lava = new PbrMaterial().baseColor([0.1, 0.05, 0.0, 1.0]).emissive([1.5, 0.4, 0.1]);