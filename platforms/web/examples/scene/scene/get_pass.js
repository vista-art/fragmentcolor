import { Pass, Scene } from "fragmentcolor";

const scene = new Scene();
scene.addPass(new Pass("backdrop"));
scene.addPass(new Pass("geometry"));

const second = scene.getPass(1).expect("two passes were added");
second.loadPrevious();