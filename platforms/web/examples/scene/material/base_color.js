import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const red = await Material.pbr(renderer).baseColor([1.0, 0.2, 0.2, 1.0]);