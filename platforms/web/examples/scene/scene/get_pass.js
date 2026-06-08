import { Pass, Scene } from "fragmentcolor";

const scene = new Scene();
scene.addPass(new Pass("backdrop"));
scene.addPass(new Pass("geometry"));

// Fetch the second pass (index 1) to reconfigure it. An out-of-range
// index returns null instead.
const geometry = scene.getPass(1);