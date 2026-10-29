import { Material, Mesh, Model, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]), Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]), ]);
const scene = new Scene();
scene.add(new Model(mesh, Material.pbr()));

scene.noDefaultCamera();