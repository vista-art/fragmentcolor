import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, { PLRender } from "../pkg/plrender.js";
await load_wasm();

const plr = new PLRender();

// The Scene is the main building
const scene = plr.Scene();

const canvas = document.getElementById("output_canvas");
const target = plr.Target({
  source: scene,

  //
  targets: canvas,

  // optional: defaults to element size
  width: canvas.width,
  height: canvas.height,

  // optional: defaults to fully transparent
  clear_color: "#00000000", // supports any CSS color string

  // defaults to fullscreen
  rect: {
    x: 0,
    y: 0,
    width: canvas.width,
    height: canvas.height,
  },

  frequency: 60, // optional: defaults to 60
});

// or scene.addTarget({ type: "canvas", selector: "#output_canvas" });
// supports: canvas (web only), window (native only), file (native only), callback
// examples:
// scene.addWindowTarget({ height: 1000, width: 800, name: "Window Title" });
// scene.addCallbackTarget({
//   height: 1000,
//   width: 800,
//   callback: (image_rgb) => {
//     console.log(image_rgb);
//   },
// });

// Create a display entity with a video texture
const video = document.getElementById("video");
const videoDisplay = new Display({ source: video }); // fullscreen by default
// Supports: ImageBitmap, ImageData, HTMLVideoElement, VideoFrame, HTMLCanvasElement, OffscreenCanvas, URL
// example: const videoDisplay = new Display({ source: "https://example.com/video.mp4" });
// Options: position, size, rotation, flip, opacity, undistorted, hidden, group

scene.add(
  plr.Display({
    source: video,
  })
);
// scene.removeEntity('World Video');

// applies lens correction; the user does not need to update it every frame
const { camera_matrix, distortion_coefficients } = undistortParams();
videoDisplay.undistort({ camera_matrix, distortion_coefficients });
// videoDisplay.undistorted = true; // enables undistortion, original image is preserved
// videoDisplay.undistorted = false; // disables undistortion, displays original distorted image

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
// scene.swapOrder(gaze, videoDisplay);
// scene.setOrder([gaze, videoDisplay]);

// Starts the event loop
// it will fail if we don't have at least one scene to render
plr.run();

// Updates gaze position every video frame.
function updateLoop() {
  const currentTime = video.currentTime;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  videoDisplay.update(video);
  scene.update();

  video.requestVideoFrameCallback(updateLoop);
}
// Actual video frame rate, Chrome/Webkit only.
// Alternatively, use requestAnimationFrame(updateLoop) for 60fps.
video.requestVideoFrameCallback(updateLoop);
