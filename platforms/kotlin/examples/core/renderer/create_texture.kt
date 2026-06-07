import org.fragmentcolor.*
val renderer = Renderer()
val png: ByteArray = byteArrayOf()
val image = "/healthcheck/public/favicon.png"
val tex = renderer.createTexture(TextureInputMobile.Path(image), null)