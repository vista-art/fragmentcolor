
import FragmentColor

let renderer = Renderer()
let canvas = document.createElement('canvas')
let target = try await renderer.createTarget(canvas)
let shader = Shader.default()

let pass = Pass("First Pass")
pass.addShader(shader)

let pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
renderer.render(pass, target)

// using a Frame
let frame = Frame()
frame.addPass(pass)
frame.addPass(pass2)
renderer.render(frame, target)

// vector of passes (consume them)
renderer.render([pass, pass2], target)