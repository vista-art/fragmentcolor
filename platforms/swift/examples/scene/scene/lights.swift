import FragmentColor

let scene = try Scene.load("path/to/model.glb")

// Darken every loaded light to half intensity for a moody pass.
for light in scene.lights() {
    let current = light.intensity()
    try light.setIntensity(current * 0.5)
}