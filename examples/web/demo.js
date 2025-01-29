import { Shader, Renderer } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
const res = [canvas.width, canvas.heigth];

// Example without config file, no defaults are set in this case
const shader = new Shader("circle.wgsl");
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
