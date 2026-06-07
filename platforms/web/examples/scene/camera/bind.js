import { Camera, Material, Renderer } from "fragmentcolor";

const camera = Camera.perspective(60.0.toRadians(), 16.0 / 9.0, 0.1, 100.0).lookAt([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

const renderer = new Renderer();
const material = await Material.pbr(renderer);
camera.bind(material.shader());
