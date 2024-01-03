import load_wasm, { FragmentColor } from "./pkg/fragmentcolor.js";
import { Scene, Sprite, Circle } from "./pkg/fragmentcolor/entities.js";

const scene = new Scene();

const worldVideo = new Sprite({ source: "#input_video" });
const gaze = new Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
});

scene.add(worldVideo);
scene.add(gaze);

scene.addTarget({ selector: "#output_canvas" });

fragmentcolor.run();
