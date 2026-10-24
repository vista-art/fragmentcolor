import FragmentColor

let m = Mesh()
try m.addInstance(Instance().set("tint", [1.0, 0.0, 0.0, 1.0]))
m.clearInstances() // back to a single uninstanced draw