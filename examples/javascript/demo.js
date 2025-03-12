import { Shader, FragmentColor as FC } from "fragmentcolor";

let canvas = document.getElementById("my-canvas");
let [renderer, target] = FC.init(canvas);
const resolution = [canvas.width, canvas.heigth];

// vertex shader is optional; the library provides a default fullscreen triangle
// and auto-detects entry points marked a @fragment, @vertex, or called main, fs_main, or vs_main
const shader = new Shader("circle.wgsl");

// The library parses the uniforms automatically and exposes their names as keys
shader.set("resolution", resolution);
shader.set("circle.radius", 200.0);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);
shader.set("circle.border", 20.0);

function animate() {
  shader.set("circle.position", [mouseX, mouseY]);
  renderer.render(shader, target); // simple example usage, it can also accept Frame objects containing multiple render passes

  requestAnimationFrame(animate);
}
animate();
