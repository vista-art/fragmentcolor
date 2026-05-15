import FragmentColor

let camera = Camera.perspective(60.0.toRadians(), 1.0, 0.1, 100.0)

// Window resize: 1920×1080 → wide-screen aspect.
camera.setAspect(1920.0 / 1080.0)