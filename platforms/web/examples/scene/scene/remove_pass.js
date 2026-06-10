import { Pass, Scene } from "fragmentcolor";

const scene = new Scene();
const backdrop = new Pass("backdrop");
const overlay = new Pass("overlay");
scene.addPass(backdrop);
scene.addPass(overlay);

// Drop the backdrop; the overlay stays.
const removed = scene.removePass(backdrop);