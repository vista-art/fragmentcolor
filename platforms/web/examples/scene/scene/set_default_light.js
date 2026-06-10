import { Light, Material, Mesh, Model, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0]), );
const scene = new Scene();
scene.add(new Model(mesh, Material.pbr()));

const key = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
scene.setDefaultLight(key);