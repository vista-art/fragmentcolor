import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, { PLRender } from "../pkg/plrender.js";
await load_wasm();

const plr = new PLRender();

const scene = plr.Scene();
const target = plr.Target({
  source: scene,
  target: "#output_canvas",
});

const backgroundVideo = plr.Display({ source: "#video" });
scene.add(backgroundVideo);

// applies lens correction; the user does not need to update it every frame
const { camera_matrix, distortion_coefficients } = undistortParams();
backgroundVideo.undistort({ camera_matrix, distortion_coefficients });

// The circle renders on top of the video because it is created later
const gaze = scene.add(
  new Circle({
    color: "#ff000088",
    radius: 0.05,
    border: 0.01,
    position: { x: 0.5, y: 0.5 },
  })
);
// undistorts position only, does not warp the circle
gaze.undistortPosition({
  camera_matrix,
  distortion_coefficients,
});

// Scene objects can be reordered or hidden
// gaze.moveToBack();
// gaze.moveToFront();
// gaze.moveForward();
// gaze.moveBackward();
// gaze.hide();
// gaze.show();
//
// Order can be set manually too:
// scene.swapOrder(gaze, backgroundVideo);
// scene.setOrder([gaze, backgroundVideo]);

// Starts the event loop
// it will fail if we don't have at least one scene to render
plr.run();

// Updates gaze position every video frame.
function updateLoop() {
  const currentTime = video.currentTime;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  backgroundVideo.update(video);
  scene.update();

  video.requestVideoFrameCallback(updateLoop);
}
// Actual video frame rate, Chrome/Webkit only.
// Alternatively, use requestAnimationFrame(updateLoop) for 60fps.
video.requestVideoFrameCallback(updateLoop);
