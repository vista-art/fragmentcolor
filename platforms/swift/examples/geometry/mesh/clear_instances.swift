import FragmentColor

let m = Mesh()
let red = [1.0, 0.0, 0.0, 1.0]
try m.addInstance(Instance().set("tint", red))
m.clearInstances(); // back to a single uninstanced draw