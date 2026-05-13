import { Material, Mesh, Model, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.0, 0.0]) .set(Vertex.NORMAL, [0.0, 1.0, 0.0]) .set(Vertex.UV0, [0.0, 0.0]), );

const model = new Model(mesh, Material.pbr()?);
model.translate([5.0, 0.0, -2.0]);
