import { Frame, Pass } from "fragmentcolor";

const pass1 = new Pass("first");
const pass2 = new Pass("second");

const frame = new Frame();
frame.add_pass(pass1);
frame.add_pass(pass2);