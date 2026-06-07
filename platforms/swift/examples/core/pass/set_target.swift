import FragmentColor

let r = Renderer()
let tex_target = try await r.createTextureTarget([512, 512])

let p = Pass("shadow")
try p.setTarget(tex_target)