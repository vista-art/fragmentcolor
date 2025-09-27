import { Frame, Pass } from "fragmentcolor";

const p1 = new Pass("shadow");
const p2 = new Pass("main");

const frame = new Frame();
f.addPass(p1);
f.addPass(p2);

frame.connect(p1, p2);