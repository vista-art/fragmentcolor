import { Frame, Pass } from "fragmentcolor";

// Example 1: simple dependency via require (lighting depends on depth)
const depth_pass = new Pass("depth_prepass");
const lighting_pass = new Pass("lighting").require(depth_pass);

// You can still use Frame for sequential pipelines
const frame1 = new Frame();
frame1.addPass(depth_pass);
frame1.addPass(lighting_pass);

// Example 2: chain using require
const geometry_pass = new Pass("geometry");
const shadow_pass = new Pass("shadows").require(geometry_pass);
const lighting_pass2 = new Pass("lighting").require(shadow_pass);
const post_process = new Pass("post_processing").require(lighting_pass2);

const frame2 = new Frame();
frame2.addPass(geometry_pass);
frame2.addPass(shadow_pass);
frame2.addPass(lighting_pass2);
frame2.addPass(post_process);
