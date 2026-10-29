import FragmentColor
import Foundation
let image = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let mesh = Mesh()
try mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]).set(Vertex.UV0, [0.5, 1.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]).set(Vertex.UV0, [0.0, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]).set(Vertex.UV0, [1.0, 0.0]),
])

// The material keeps the encoded image as a pending upload.
let material = Material.pbr().baseColorTexture(image)
let model = Model(mesh, material)
let scene = Scene()
try scene.add(model)

// Decode and upload it now, so the first render only draws.
try await renderer.load(scene)
try renderer.render(scene, target)