import { PbrMaterial } from "fragmentcolor";

// Every map spans the surface twice, and the normal map eight times.
const tiles = new PbrMaterial().uvTransform([0.0, 0.0], [2.0, 2.0], 0.0);
const normals = tiles.normalMap().transform([0.0, 0.0], [8.0, 8.0], 0.0);