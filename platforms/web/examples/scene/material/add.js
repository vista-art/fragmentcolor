import { Camera, Light, Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const material = await Material.pbr(renderer);

const camera = Camera.perspective(60.0.toRadians(), 16.0 / 9.0, 0.1, 100.0).lookAt([0.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
const sun = Light.directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);

material.add(camera).add(sun);

// Updating the camera later is enough — the Material picks the new
// view_proj up at the next render without re-adding.
camera.lookAt([3.0, 1.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);