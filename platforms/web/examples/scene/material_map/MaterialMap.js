import { PbrMaterial } from "fragmentcolor";

const flat = [128, 128, 255, 255];
const gold = new PbrMaterial().metallic(1.0).roughness(0.3);

// A normal map tiled four times over the second UV layout.
const map = gold.normalMap().texture(flat, [1, 1]).uv(1).transform([0.0, 0.0], [4.0, 4.0], 0.0);