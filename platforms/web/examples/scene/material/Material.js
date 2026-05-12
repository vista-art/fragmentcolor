import { Material, Mesh, Model, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]), );

const material = Material.pbr().baseColor([0.85, 0.2, 0.2, 1.0]).metallic(0.0).roughness(0.4).emissive([0.0, 0.0, 0.05]);

const model = new Model(mesh, material);