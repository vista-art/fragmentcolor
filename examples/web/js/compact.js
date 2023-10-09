import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, { PLRender } from "../pkg/plrender.js";
await load_wasm();

const plr = new PLRender();

const scene = plr.Scene();
const target = plr.Target({
  source: scene,
  target: "#output_canvas",
});

const backgroundVideo = plr.Background({ source: "#video" });
scene.add(backgroundVideo);

const { camera_matrix, distortion_coefficients } = undistortParams();
backgroundVideo.undistort({ camera_matrix, distortion_coefficients });

const gaze = plr.Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
});
scene.add(gaze);

gaze.undistortPosition({
  camera_matrix,
  distortion_coefficients,
});

plr.run();

function updateLoop() {
  const currentTime = video.currentTime;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  backgroundVideo.update(video);
  scene.update();

  video.requestVideoFrameCallback(updateLoop);
}
video.requestVideoFrameCallback(updateLoop);
