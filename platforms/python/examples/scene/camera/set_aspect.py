from fragmentcolor import Camera

camera = Camera.perspective(60.0_.to_radians(), 1.0, 0.1, 100.0)

# Window resize: 1920×1080 → wide-screen aspect.
camera.set_aspect(1920.0 / 1080.0)