
import FragmentColor
let renderer = Renderer()

// Create an offscreen texture target with a size of 64x64 pixels.
let target = try await renderer.createTextureTarget([64, 64])

renderer.render(Shader(""), target)

// get the rendered image
let image = target.getImage()