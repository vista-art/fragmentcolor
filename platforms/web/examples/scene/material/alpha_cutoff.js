import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const foliage = await Material.pbr(renderer).alphaCutoff(0.3);