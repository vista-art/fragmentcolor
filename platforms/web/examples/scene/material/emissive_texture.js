import { Material } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const glow_bytes = [ 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, ];
const mat = Material.pbr().emissive([0.8, 0.0, 0.0]).emissiveTexture(glow_bytes, [2, 2]);