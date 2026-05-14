from fragmentcolor import Material

# Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
brick = Material.pbr().uv_transform([0.5, 0.0], [4.0, 4.0], std.f32.consts.FRAC_PI_4)