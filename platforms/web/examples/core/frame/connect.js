import { Frame, Pass } from "fragmentcolor";

// Example 1: simple dependency via require (lighting depends on depth)
const depth = new Pass("depth_prepass");
const lighting = new Pass("lighting");
lighting.require(depth);

// You can still use Frame for sequential pipelines
const frame1 = new Frame();
frame1.addPass(depth);
frame1.addPass(lighting);

// Example 2: chain using require
const geometry = new Pass("geometry");
const shadow = new Pass("shadows");
shadow.require(geometry);
const lighting2 = new Pass("lighting");
lighting2.require(shadow);
const post_process = new Pass("post_processing");
post_process.require(lighting2);

const frame2 = new Frame();
frame2.addPass(geometry);
frame2.addPass(shadow);
frame2.addPass(lighting2);
frame2.addPass(post_process);
