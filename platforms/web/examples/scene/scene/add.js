import { Camera, Light, Material, Mesh, Model, Renderer, Scene, Vertex } from "fragmentcolor";

const renderer = new Renderer();

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]) .set(Vertex.COLOR0, [1.0, 1.0, 1.0, 1.0]) .set(Vertex.UV1, [0.0, 0.0]).set(Vertex.TANGENT, [1.0, 0.0, 0.0, 1.0]), );
const model = new Model(mesh, Material.pbr()?);

const camera = Camera.perspective(60.0.toRadians(), 1.0, 0.1, 100.0).lookAt([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
const sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

const scene = new Scene();
scene.add(model)?.add(camera)?.add(sun);

// Updating the camera later is enough — every shader on the scene picks
// the new view_proj up at the next render.
camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);