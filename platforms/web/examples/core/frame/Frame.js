import { Frame, Pass } from "fragmentcolor";

const pass1 = new Pass("first");
const pass2 = new Pass("second");

const frame = new Frame();
frame.addPass(pass1);
frame.addPass(pass2);