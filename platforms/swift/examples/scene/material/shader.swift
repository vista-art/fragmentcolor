import FragmentColor

let material = Material.pbr()
try material.shader().set(
    "camera.viewProj",
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ],
)
try material.shader().set("camera.position", [0.0, 0.0, 5.0])