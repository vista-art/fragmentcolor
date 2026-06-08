import { Material, Mesh, Model, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0]), );
const scene = new Scene();
scene.add(new Model(mesh, Material.pbr()));

// Compose, don't clear: keep whatever the previous pass drew.
for (const pass of scene.listPasses()) { pass.loadPrevious(); };