import org.fragmentcolor.*

// Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
val brick = Material.pbr()?.uvTransform(listOf(0.5f, 0.0f), listOf(4.0f, 4.0f), std.f32.consts.FRACPI4)