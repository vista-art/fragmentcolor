import { Scene, SceneSource } from "fragmentcolor";

const scene = Scene.load(SceneSource.gltf("path/to/model.glb"));

// glTF shipped a camera — animate it per frame instead of supplying our own.
if let Some(camera) = scene.cameras().intoIter().next() { camera.lookAt([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]); camera.setAspect(16.0 / 9.0); };