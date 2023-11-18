import load_wasm, { PLRender } from "./pkg/plrender.js";
import { Scene, Sprite, Circle } from "./pkg/plrender/entities.js";

const scene = new Scene();

const worldVideo = new Sprite({ source: "#input_video" });
const gaze = new Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
  position: [0.5, 0.5],
});

scene.add(worldVideo);
scene.add(gaze);

scene.renderTo({ selector: "#output_canvas" });

plrender.run();
