import { Light, Material, Mesh, Model, Renderer, Scene, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const mesh = new Mesh();
mesh.addVertex( Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]), );

const scene = new Scene();
// Warm dusk ambient — applies to every Material added below.
scene.ambient([0.06, 0.04, 0.03]);
scene.add(Model.new(mesh, Material.pbr()?));
scene.add(Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]));

renderer.render(scene, target);