import { Frame, Pass } from "fragmentcolor";

const shadow = new Pass("shadow");
const main = new Pass("main");

const frame = new Frame();
frame.addPass(shadow);
frame.addPass(main);
main.require(shadow);
