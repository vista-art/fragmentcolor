import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, { PLRender } from "../pkg/plrender.js";
await load_wasm();

const plr = new PLRender();

// The Scene is a container of renderable entities
// It manages the spatial relationship between them
const scene = plr.Scene();

const canvas = document.getElementById("output_canvas");
const target = plr.Target({
  source: scene,

  // optional: defaults to empty array []
  // supports: QuerySelector, CanvasElement, OffscreenCanvas
  targets: [canvas],

  // optional: defaults to first element's CSS size
  // units are in pixels
  width: canvas.width,
  height: canvas.height,

  // optional: defaults to fully transparent
  clear_color: "#00000000", // supports any CSS color string

  // optionsl: defaults to fullscreen
  // this is the region of the target to draw the scene
  viewport: {
    x: 0,
    y: 0,
    width: canvas.width,
    height: canvas.height,
  },

  // optional: defaults to screen's refresh rate
  // in the web, this is the frequency of the requestAnimationFrame loop
  fps: 60,

  // optional: defaults to empty function
  before_render: () => {
    // if you want to synchronize the scene state
    // use this callback to update any scene object
    // right before render,
    //  with the rendering loop
    // do something before rendering the scene
  },

  // optional: defaults to empty function
  after_render: (frame) => {
    // frame is a ImageBitmap
  },
});

// Create a display entity with a video texture
const video = document.getElementById("video");
const videoBackground = new Background({ source: video }); // fullscreen by default
// Supports: ImageBitmap, ImageData, HTMLVideoElement, VideoFrame, HTMLCanvasElement, OffscreenCanvas, URL
// example: const videoBackground = new Background({ source: "https://example.com/video.mp4" });
// Options: size, rotation, flip, opacity, undistorted, hidden, group
// Background is a subclass of Display with position locked to (0, 0, camera_far_plane)

scene.add(
  plr.Background({
    source: video,
  })
);
// scene.removeEntity('World Video');

// applies lens correction; the user does not need to update it every frame
const { camera_matrix, distortion_coefficients } = undistortParams();
videoBackground.undistort({ camera_matrix, distortion_coefficients });
// videoBackground.undistorted = true; // enables undistortion, original image is preserved
// videoBackground.undistorted = false; // disables undistortion, displays original distorted image

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
// scene.swapOrder(gaze, videoBackground);
// scene.setOrder([gaze, videoBackground]);

// Starts the event loop
// it will fail if we don't have at least one scene to render
plr.run();

// Updates gaze position every video frame.
function updateLoop() {
  const currentTime = video.currentTime;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  videoBackground.update(video);
  scene.update();

  video.requestVideoFrameCallback(updateLoop);
}
// Actual video frame rate, Chrome/Webkit only.
// Alternatively, use requestAnimationFrame(updateLoop) for 60fps.
video.requestVideoFrameCallback(updateLoop);
