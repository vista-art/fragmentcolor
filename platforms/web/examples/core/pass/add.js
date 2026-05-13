import { Camera, Light, Material, Mesh, Model, Pass, Renderer, Vertex } from "fragmentcolor";

const renderer = new Renderer();

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]), );
const model = await new Model(mesh, Material.pbr(renderer));

const camera = Camera.perspective(60.0.toRadians(), 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
const sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

const pass = new Pass("scene");
pass.addModel(model);
pass.add(camera).add(sun);

// Updating the camera later is enough — every Model already on the pass
// picks the new view_proj up at the next render.
camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);