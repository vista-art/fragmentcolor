import { Material, Mesh, Model, Renderer, Scene, Vertex } from "fragmentcolor";
const image = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const mesh = new Mesh();
mesh.addVertices([ Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]), Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]), Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]), ]);

// The material keeps the encoded image as a pending upload.
const material = Material.pbr().baseColorTexture(image);
const model = new Model(mesh, material);
const scene = new Scene();
scene.add(model);

// Decode and upload it now, so the first render only draws.
await renderer.load(scene);
renderer.render(scene, target);