import { Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const wall = new Mesh();
wall.addVertices([ Vertex.pbr([-2.0, -2.0, 0.0]), Vertex.pbr([2.0, -2.0, 0.0]), Vertex.pbr([2.0, 2.0, 0.0]), Vertex.pbr([-2.0, 2.0, 0.0]), ]);
wall.setIndices([0, 1, 2, 0, 2, 3]);

const camera = Camera.perspective(1.047, 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
const bulb = Light.point([0.0, 0.5, 1.0], [1.0, 0.95, 0.8]).setIntensity(15.0);
bulb.setRange(3.0);

const pass = new Pass("lamp");
pass.add(new Model(wall, Material.pbr()));
pass.add(camera);
pass.add(bulb);
renderer.render(pass, target);
