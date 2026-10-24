import FragmentColor

let m = Mesh()
m.addInstances([
    try Instance().set("tint", [1.0, 0.0, 0.0, 1.0]),
    try Instance().set("tint", [0.0, 1.0, 0.0, 1.0]),
    try Instance().set("tint", [0.0, 0.0, 1.0, 1.0]),
])