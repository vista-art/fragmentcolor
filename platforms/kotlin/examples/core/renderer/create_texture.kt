import org.fragmentcolor.*
val renderer = Renderer()
// Load encoded image bytes (PNG/JPEG) or use a file path
val image = "/healthcheck/public/favicon.png"
val tex = renderer.createTexture(image)