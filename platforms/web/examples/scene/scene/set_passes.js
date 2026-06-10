import { Pass, Scene } from "fragmentcolor";

const scene = new Scene();
scene.addPass(new Pass("scratch"));

// Swap in a deliberate order: shadow map, then geometry, then overlay.
scene.setPasses([ new Pass("shadow"), new Pass("geometry"), new Pass("overlay"), ]);