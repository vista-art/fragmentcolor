import { PbrMaterial } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const normal_map_bytes = [ 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, ];
const material = new PbrMaterial().normalScale(1.2);
const map = material.normalMap().texture(normal_map_bytes, [2, 2]);