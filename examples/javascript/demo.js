import init, { Shader, Renderer } from "fragmentcolor";

async function run() {
  await init();
  let canvas = document.getElementById("my-canvas");

  const renderer = new Renderer();
  const target = await renderer.create_target(canvas);
  const resolution = [canvas.width, canvas.height];

  // vertex shader is optional; the library provides a default fullscreen triangle
  // and auto-detects entry points marked a @fragment, @vertex, or called main, fs_main, or vs_main
  const shader = new Shader("circle.wgsl");

  // The library parses the uniforms automatically and exposes their names as keys
  shader.set("resolution", resolution);
  shader.set("circle.radius", 200.0);
  shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);
  shader.set("circle.border", 20.0);

  function animate() {
    shader.set("circle.position", [0.0, 0.0]);
    renderer.render(shader, target); // can also accept a Pass, or an array of Pass objects, for multi-pass rendering

    requestAnimationFrame(animate);
  }
  animate();
}
run();
