import FragmentColor

let m = Mesh()
let red = [1.0, 0.0, 0.0, 1.0]
let green = [0.0, 1.0, 0.0, 1.0]
let blue = [0.0, 0.0, 1.0, 1.0]
m.addInstances([
    try Instance().set("tint", red),
    try Instance().set("tint", green),
    try Instance().set("tint", blue),
])