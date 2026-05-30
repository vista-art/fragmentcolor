import { Scene } from "fragmentcolor";

// A path — `.gltf` JSON (with external buffers/images) or a `.glb` container.
const scene = Scene.load("path/to/model.gltf");

// In-memory `.glb` bytes — from disk, the network, or another asset pipeline.
const glb_bytes = "/healthcheck/public/favicon.png";
const scene2 = Scene.load(glb_bytes);