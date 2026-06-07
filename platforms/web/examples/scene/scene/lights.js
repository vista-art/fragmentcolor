import { Scene } from "fragmentcolor";

const response = await fetch("/healthcheck/public/model.glb");
const bytes = new Uint8Array(await response.arrayBuffer());
const scene = Scene.load(bytes);

// Darken every loaded light to half intensity for a moody pass.
for (const light of scene.lights()) {
  const current = light.intensity();
  light.setIntensity(current * 0.5);
}