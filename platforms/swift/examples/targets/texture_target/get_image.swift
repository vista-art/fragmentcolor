import FragmentColor


let renderer = Renderer()
let target = try await renderer.createTextureTarget([16, 16])
renderer.render(Shader(""), target)

let image = target.getImage()