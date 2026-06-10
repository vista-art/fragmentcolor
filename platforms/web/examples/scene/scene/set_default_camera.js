import { Camera, Material, Mesh, Model, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0]), );
const scene = new Scene();
scene.add(new Model(mesh, Material.pbr()));

const camera = Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).lookAt([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
scene.setDefaultCamera(camera);