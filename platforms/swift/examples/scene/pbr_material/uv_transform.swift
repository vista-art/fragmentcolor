import FragmentColor

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
let brick = try PbrMaterial().uvTransform([0.5, 0.0], [4.0, 4.0], 0.785)