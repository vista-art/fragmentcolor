import org.fragmentcolor.*
val renderer = Renderer()
// Encoded image bytes (PNG / JPEG / etc.) â single tuple, no extra method.
val image = "/healthcheck/public/favicon.png"
val tex = renderer.createTexture(TextureInputMobile.Path(image), null)