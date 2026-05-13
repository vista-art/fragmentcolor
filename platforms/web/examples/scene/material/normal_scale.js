import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const detailed = await Material.pbr(renderer).normalScale(1.5);