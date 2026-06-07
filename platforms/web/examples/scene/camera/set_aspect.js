import { Camera } from "fragmentcolor";

const camera = Camera.perspective(1.047, 1.0, 0.1, 100.0);

// Window resize: 1920×1080 → wide-screen aspect.
camera.setAspect(1920.0 / 1080.0);