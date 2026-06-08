import FragmentColor

let scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

// Fetch the second pass (index 1) to reconfigure it. An out-of-range
// index returns nil instead.
let geometry = scene.getPass(1)