import { Material, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const glow = await renderer.createTexture([ 255, 0, 0, 255, 255,   0, 0, 255, 255,   0, 0, 255, 255,   0, 0, 255, ][..]);
const mat = Material.pbr().emissive([0.8, 0.0, 0.0]).emissiveTexture(glow);