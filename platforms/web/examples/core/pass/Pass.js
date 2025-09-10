
import { Shader, Pass, Renderer, Frame } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([10, 10]);
const shader = Shader.default();

const pass = new Pass("First Pass");
pass.addShader(shader);

const pass2 = new Pass("Second Pass");
pass2.addShader(shader);

// standalone;
renderer.render(pass, target);

// using a Frame;
const frame = new Frame();
frame.addPass(pass);
frame.addPass(pass2);
renderer.render(frame, target);

// vector of passes (consume them);
renderer.render([pass, pass2], target);
