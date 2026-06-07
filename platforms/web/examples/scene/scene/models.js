import { Material, Mesh, Model, Scene, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set("uv0", [0.5, 1.0]), );
const model = new Model(mesh, Material.pbr());

const scene = new Scene();
scene.add(model);

// LOD switch: hide every model the user just loaded, based on a
// camera-distance heuristic the caller computes elsewhere.
for (const m of scene.models()) { m.setVisible(false); };