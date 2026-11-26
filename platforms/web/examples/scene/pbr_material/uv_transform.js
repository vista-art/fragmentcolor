import { PbrMaterial } from "fragmentcolor";

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
const brick = new PbrMaterial().uvTransform([0.5, 0.0], [4.0, 4.0], 0.785);