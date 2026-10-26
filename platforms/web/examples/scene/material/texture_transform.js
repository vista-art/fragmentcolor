import { Material, TextureSlot } from "fragmentcolor";

// Every map spans the surface twice, and the normal map eight times.
const tiles = Material.pbr().uvTransform([0.0, 0.0], [2.0, 2.0], 0.0).textureTransform(TextureSlot.Normal, [0.0, 0.0], [8.0, 8.0], 0.0);