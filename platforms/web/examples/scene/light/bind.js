import { Light, Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const material = await Material.pbr(renderer);
const sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
sun.bind(material.shader());
