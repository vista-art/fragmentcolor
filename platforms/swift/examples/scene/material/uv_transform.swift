import FragmentColor

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
let brick = Material.pbr()?.uvTransform([0.5, 0.0], [4.0, 4.0], std.f32.consts.fRACPI4)