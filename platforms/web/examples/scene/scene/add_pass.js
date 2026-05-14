import { Material, Mesh, Model, Pass, Renderer, Scene, Vertex } from "fragmentcolor";

const renderer = new Renderer();

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]) .set(Vertex.COLOR0, [1.0, 1.0, 1.0, 1.0]) .set(Vertex.UV1, [0.0, 0.0]).set(Vertex.TANGENT, [1.0, 0.0, 0.0, 1.0]), );
const model = new Model(mesh, Material.pbr()?);

// A backdrop pass that clears to a soft blue before the scene's main draw.
const backdrop = new Pass("backdrop");
backdrop.setClearColor([0.05, 0.08, 0.12, 1.0]);

const scene = new Scene();
scene.addPass(backdrop);
scene.add(model);
