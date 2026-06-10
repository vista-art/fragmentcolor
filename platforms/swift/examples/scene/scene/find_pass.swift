import FragmentColor

let scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

// Look the geometry pass up by name to reconfigure it. A name with no
// match returns nil instead.
let geometry = scene.findPass("geometry")