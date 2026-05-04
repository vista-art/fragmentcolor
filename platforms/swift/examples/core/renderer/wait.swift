import FragmentColor

let r = Renderer()
let target = try await r.createTextureTarget([8, 8])
let shader = Shader.default()
try r.render(shader, target)
r.wait()
let _bytes = try await target.getImage()