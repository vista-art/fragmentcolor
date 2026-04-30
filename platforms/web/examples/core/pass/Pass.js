
import { Shader, Pass, Renderer } from "fragmentcolor";

const renderer = new Renderer();
const canvas = document.createElement('canvas');
const target = await renderer.createTarget(canvas);
const shader = Shader.default();

const pass = new Pass("First Pass");
pass.addShader(shader);

const pass2 = new Pass("Second Pass");
pass2.addShader(shader);

// standalone
renderer.render(pass, target);

// vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render([pass, pass2], target);
