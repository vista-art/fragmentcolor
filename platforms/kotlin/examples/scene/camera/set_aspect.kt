import org.fragmentcolor.*

val camera = Camera.perspective(1.047f, 1.0f, 0.1f, 100.0f)

// Window resize: 1920×1080 → wide-screen aspect.
camera.setAspect(1920.0f / 1080.0f)