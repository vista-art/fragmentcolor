
import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([100, 100])

let pass1 = Pass("first")
let pass2 = Pass("second")

let frame = Frame()
frame.addPass(pass1)
frame.addPass(pass2)