import FragmentColor

let camera = try Camera.perspective(1.047, 1.0, 0.1, 100.0)

// Window resize: 1920×1080 → wide-screen aspect.
try camera.setAspect(1920.0 / 1080.0)