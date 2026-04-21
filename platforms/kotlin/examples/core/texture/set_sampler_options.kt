import org.fragmentcolor.*
val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = [255,255,255,255]

val texture = renderer.createTextureWithSize(pixels, [1,1])
val opts = {repeat_x: true, repeat_y: true, smooth: true, compare: null}
texture.setSamplerOptions(opts)
