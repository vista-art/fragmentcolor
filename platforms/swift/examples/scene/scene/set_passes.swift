import FragmentColor

let scene = Scene()
scene.addPass(Pass("scratch"))

// Swap in a deliberate order: shadow map, then geometry, then overlay.
scene.setPasses([
    Pass("shadow"),
    Pass("geometry"),
    Pass("overlay"),
])