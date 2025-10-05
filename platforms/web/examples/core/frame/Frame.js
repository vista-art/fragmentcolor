
import { Shader, Pass, Renderer, Frame } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([100, 100]);

const pass1 = new Pass("first");
const pass2 = new Pass("second");

const frame = new Frame();
frame.addPass(pass1);
frame.addPass(pass2);
