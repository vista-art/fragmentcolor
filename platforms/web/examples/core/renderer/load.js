import { Material, Mesh, Model, Renderer, Scene, Vertex } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const mesh = new Mesh();
mesh.addVertex( Vertex.new([0.0, 0.5, 0.0]) .set(Vertex.NORMAL, [0.0, 0.0, 1.0]) .set(Vertex.UV0, [0.5, 1.0]), );
// Raw 2×2 RGBA pixel bytes — uploaded lazily by `Renderer.load` below.
// In practice the loader hands the setter encoded PNG/JPEG bytes (from a
// BIN chunk) or a file path (from a URI); the same `Into<TextureInput>`
// vocabulary covers all of them.
const red_pixels = [ 255,   0,   0, 255,    0, 255,   0, 255, 0,   0, 255, 255,  255, 255, 255, 255, ];
const material = Material.pbr()?.baseColorTexture(red_pixels, [2, 2]);
const model = new Model(mesh, material);
const scene = new Scene();
scene.add(model);

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
await renderer.load(scene);
renderer.render(scene, target);