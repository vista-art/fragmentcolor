import { Scene, SceneSource } from "fragmentcolor";

// File path — covers both `.gltf` JSON (with external buffers/images)
// and `.glb` binary containers.
const scene = Scene.load(SceneSource.gltf("path/to/model.gltf"));

// In-memory `.glb` bytes — fetched from disk, network, or a BIN chunk
// in another format.
const glb_bytes = [/* … */];
const scene2 = Scene.load(SceneSource.gltf(glb_bytes));