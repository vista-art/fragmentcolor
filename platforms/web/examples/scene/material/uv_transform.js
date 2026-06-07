import { Material } from "fragmentcolor";

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
const brick = Material.pbr().uvTransform([0.5, 0.0], [4.0, 4.0], 0.785); // 45° in radians;