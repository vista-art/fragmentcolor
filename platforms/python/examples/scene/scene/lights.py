from fragmentcolor import Scene

scene = Scene.load("path/to/model.glb")

# Darken every loaded light to half intensity for a moody pass.
for light in scene.lights():
    current = light.intensity()
    light.set_intensity(current * 0.5)