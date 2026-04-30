import FragmentColor

let r = Renderer()
let target = try await r.createTextureTarget([8, 8])
let shader = Shader.default()
r.render(shader, target)
r.waitIdle()
let _bytes = target.getImage()