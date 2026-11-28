import { PbrMaterial } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const ao_bytes = [ 220, 0, 0, 255, 180, 0, 0, 255, 200, 0, 0, 255, 160, 0, 0, 255, ];
const material = new PbrMaterial();
const map = material.occlusionMap().texture(ao_bytes, [2, 2]);