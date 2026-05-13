import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const foliage = Material.pbr()?.alphaCutoff(0.3);