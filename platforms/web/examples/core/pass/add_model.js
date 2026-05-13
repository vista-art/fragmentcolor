import { Material, Mesh, Model, Pass, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]), );
mesh.addVertex( Vertex.new([-0.5, -0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.0, 0.0]), );
mesh.addVertex( Vertex.new([0.5, -0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [1.0, 0.0]), );

const template = await Material.pbr(renderer).baseColor([0.85, 0.4, 0.2, 1.0]);
const pass = new Pass("scene");

const m1 = new Model(mesh.clone(), template.clone());
m1.translate([-1.0, 0.0, 0.0]);
pass.addModel(m1);

const m2 = new Model(mesh, template);
m2.translate([1.0, 0.0, 0.0]);
pass.addModel(m2);
