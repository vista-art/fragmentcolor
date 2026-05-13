import { Material, Mesh, Model, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.0, 0.0]) .set(Vertex.NORMAL, [0.0, 1.0, 0.0]) .set(Vertex.UV0, [0.0, 0.0]), );

const model = new Model(mesh, Material.pbr()?);
model.scale([2.0, 2.0, 2.0]);