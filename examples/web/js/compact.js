import { positionForTime /*undistortParams*/ } from "./mocks.js";
import load_wasm, { Scene, Circle, Sprite } from "../pkg/fragmentcolor.js";
await load_wasm();

const scene = new Scene();
scene.addTarget("#outputCanvas");

const background = Sprite({ source: "#video" });
scene.add(background);

const { cameraMatrix, distortionCoefficients } = undistortParams();
background.undistort({ cameraMatrix, distortionCoefficients });

const gaze = Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
});
scene.add(gaze);

gaze.undistortPosition({
  cameraMatrix,
  distortionCoefficients,
});

scene.run();

function updateLoop() {
  const currentTime = video.currentTime;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  background.update(video);
  scene.update();

  video.requestVideoFrameCallback(updateLoop);
}
video.requestVideoFrameCallback(updateLoop);
