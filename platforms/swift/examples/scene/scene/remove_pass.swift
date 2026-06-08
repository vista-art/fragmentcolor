import FragmentColor

let scene = Scene()
let backdrop = Pass("backdrop")
let overlay = Pass("overlay")
scene.addPass(backdrop)
scene.addPass(overlay)

// Drop the backdrop; the overlay stays.
let removed = scene.removePass(backdrop)