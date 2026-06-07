import { Scene } from "fragmentcolor";

// Fetch the `.glb` container as bytes, then load it. The same call accepts
// a path string on native; on web pass the bytes instead.
const response = await fetch("/healthcheck/public/model.glb");
const bytes = new Uint8Array(await response.arrayBuffer());
const scene = Scene.load(bytes);