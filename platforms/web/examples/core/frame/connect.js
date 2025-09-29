import { Frame, Pass } from "fragmentcolor";

const depth_pass = new Pass("depth_prepass");
const lighting_pass = new Pass("lighting");

const frame = new Frame();
frame.addPass(depth_pass);
frame.addPass(lighting_pass);

// Ensure depth prepass runs before lighting;
frame.connect(depth_pass, lighting_pass);
import { Frame, Pass } from "fragmentcolor";

const geometry_pass = new Pass("geometry");
const shadow_pass = new Pass("shadows");
const lighting_pass = new Pass("lighting");
const post_process = new Pass("post_processing");

const frame = new Frame();

// Add all passes;
frame.addPass(geometry_pass);
frame.addPass(shadow_pass);
frame.addPass(lighting_pass);
frame.addPass(post_process);

// Build dependency chain;
frame.connect(geometry_pass, shadow_pass);
frame.connect(shadow_pass, lighting_pass);
frame.connect(lighting_pass, post_process);