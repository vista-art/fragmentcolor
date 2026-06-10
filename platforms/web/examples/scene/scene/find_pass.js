import { Pass, Scene } from "fragmentcolor";

const scene = new Scene();
scene.addPass(new Pass("backdrop"));
scene.addPass(new Pass("geometry"));

// Look the geometry pass up by name to reconfigure it. A name with no
// match returns null instead.
const geometry = scene.findPass("geometry");