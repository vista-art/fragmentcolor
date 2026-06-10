import { Material, Mesh, Model, Pass, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0]), );
const model = new Model(mesh, Material.pbr());

const scene = new Scene();
scene.addPass(new Pass("geometry"));

// Target the pass by name (or pass its index: scene.addTo(0, model)).
scene.addTo("geometry", model);