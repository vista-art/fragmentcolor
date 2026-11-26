import { PbrMaterial } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const tint = [255, 128, 0, 255];
const material = new PbrMaterial();
const map = material.baseColorMap().texture(tint, [1, 1]);