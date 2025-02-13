import { Shader, Renderer } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
const res = [canvas.width, canvas.heigth];

const shader = new Shader("circle.wgsl");

// The library parses the uniforms automatically and exposes them as keys
shader.set("resolution", res);
shader.set("circle.radius", 0.05);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);

const renderer = new Renderer();
renderer.render(shader);

function animate() {
  shader.set("circle.position", [mouseX, mouseY]);
  renderer.render(shader, canvas);

  requestAnimationFrame(animate);
}
animate();
