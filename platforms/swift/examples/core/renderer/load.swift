import FragmentColor

let renderer = Renderer()
let target = try await renderer.createTextureTarget([64, 64])

let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.5, 1.0]),
)
// Raw 2×2 RGBA pixel bytes — uploaded lazily by """Renderer.load""" below.
// In practice the loader hands the setter encoded PNG/JPEG bytes (from a
// BIN chunk) or a file path (from a URI); the same """Into<TextureInput>"""
// vocabulary covers all of them.
let red_pixels = [
    255,   0,   0, 255,    0, 255,   0, 255,
      0,   0, 255, 255,  255, 255, 255, 255,
]
let material = Material.pbr()?.baseColorTexture((red_pixels, [2, 2]))
let model = Model(mesh, material)
let scene = Scene()
scene.add(model)

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
try await renderer.load(scene)
try renderer.render(scene, target)