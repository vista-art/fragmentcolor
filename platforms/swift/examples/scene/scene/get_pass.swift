import FragmentColor

let scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

let second = scene.getPass(1).expect("two passes were added")
second.loadPrevious()