from fragmentcolor import PbrMaterial

# Tile the texture 4× in both directions, rotate 45°, shift by half a tile.
brick = PbrMaterial().uv_transform([0.5, 0.0], [4.0, 4.0], 0.785)