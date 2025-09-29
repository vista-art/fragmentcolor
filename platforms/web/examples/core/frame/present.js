import { Frame, Pass } from "fragmentcolor";

const shadow = new Pass("shadow");
const main = new Pass("main");

const frame = new Frame();
frame.addPass(shadow);
frame.addPass(main);
frame.connect(shadow, main);

// Main pass is a leaf, so it can present;
frame.present(main);
import { Frame, Pass } from "fragmentcolor";

const geometry = new Pass("geometry");
const lighting = new Pass("lighting");
const post_fx = new Pass("post_effects");

const frame = new Frame();
frame.addPass(geometry);
frame.addPass(lighting);
frame.addPass(post_fx);

// Build pipeline;
frame.connect(geometry, lighting);
frame.connect(lighting, post_fx);

// Final post-effects pass presents to screen;
frame.present(post_fx);