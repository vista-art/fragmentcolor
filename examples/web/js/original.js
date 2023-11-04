import load_wasm, { PLRender } from "./pkg/plrender.js";
import { Display, Circle } from "./pkg/plrender/entities.js";

const scene = plrender.Scene();
scene.addTarget({ selector: "#output_canvas" });

const video = document.getElementById("video");
const worldVideo = new Sprite({ source: video });
const gaze = new Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
  position: [0.5, 0.5],
});

scene.add(worldVideo);
scene.add(gaze);

plrender.run();
