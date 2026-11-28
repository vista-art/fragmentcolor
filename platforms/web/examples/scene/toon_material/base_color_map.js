import { ToonMaterial } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const pixels = [255, 128, 0, 255];
const material = new ToonMaterial();
const map = material.baseColorMap().texture(pixels, [1, 1]);