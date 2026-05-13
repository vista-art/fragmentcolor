import { Material, Mesh, Model, Pass, Vertex } from "fragmentcolor";

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]), );
mesh.addVertex( Vertex.new([-0.5, -0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.0, 0.0]), );
mesh.addVertex( Vertex.new([0.5, -0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [1.0, 0.0]), );

const template = Material.pbr()?.baseColor([0.85, 0.4, 0.2, 1.0]);
const pass = new Pass("scene");

const m1 = new Model(mesh.clone(), template.clone());
m1.translate([-1.0, 0.0, 0.0]);
pass.addModel(m1);

const m2 = new Model(mesh, template);
m2.translate([1.0, 0.0, 0.0]);
pass.addModel(m2);
