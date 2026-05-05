import FragmentColor

let m = Mesh()
let offset = [0.25, 0.10]
let tint = [1.0, 0.0, 0.0, 1.0]
try m.addInstance(Instance().set("offset", offset).set("tint", tint))