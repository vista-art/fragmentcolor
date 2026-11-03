import { Material } from "fragmentcolor";

// Raw pixels with their size upload when the scene loads.
const mr_map_bytes = [ 0, 200, 50, 255, 0, 240, 80, 255, 0, 180, 30, 255, 0, 220, 60, 255, ];
const mat = Material.pbr().metallicRoughnessTexture(mr_map_bytes, [2, 2]);