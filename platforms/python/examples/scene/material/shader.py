from fragmentcolor import Material

material = Material.pbr()
material.shader().set(
    "camera.view_proj",
    [
        [1.0_, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ],
)
material.shader().set("camera.position", [0.0_, 0.0, 5.0])