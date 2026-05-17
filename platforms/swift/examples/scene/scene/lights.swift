import FragmentColor

let scene = Scene.load(SceneSource.gltf("path/to/model.glb"))

// Darken every loaded light to half intensity for a moody pass.
for light in scene.lights() {
let current = light.intensity()
    light.setIntensity(current * 0.5)
}