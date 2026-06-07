import org.fragmentcolor.*

val scene = Scene.load("path/to/model.glb")

// Darken every loaded light to half intensity for a moody pass.
for (light in scene.lights()) {
    val current = light.intensity()
    light.setIntensity(current * 0.5f)
}