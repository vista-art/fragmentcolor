import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, { PLRender } from "../pkg/plrender.js";
await load_wasm();

// All settings are optional.
// The defaults are shown below.
const plr = new PLRender({
  log_level: "info", // or 'trace', 'debug', 'warn', 'error'
  power_preference: "high-performance", // or 'low-power', 'no-preference'
  force_software_rendering: false,
});

// The Scene is a container of renderable entities
// It manages the spatial relationship between them
const scene = plr.Scene();

// Targets are platform-specific surfaces.
// Examples: OS window, Jupyter cell, Web Canvas
const canvas = document.getElementById("output_canvas");

// A Renderer draws a scene to one or multiple targets
const renderer = plr.Renderer({
  source: scene, // required: a Scene instance

  // optional: defaults to empty array []
  // supports: QuerySelector, CanvasElement, OffscreenCanvas
  targets: [canvas],

  // optional: defaults to first element's CSS size
  // All renderer units are in pixels
  width: canvas.width,
  height: canvas.height,

  // optional: defaults to fully transparent
  clear_color: "#00000000", // supports any CSS color string

  // optionsl: defaults to fullscreen
  // this is the region of the target to draw the scene
  // you can use it to draw multiple scenes to the same target
  clip: {
    x: 0,
    y: 0,
    width: canvas.width,
    height: canvas.height,
  },

  // optional: defaults to screen's refresh rate, normally 60 hz
  // In the web, this is the same frequency as requestAnimationFrame()
  framerate: 60,
  // if you set it to 0, it will only render
  // when you call renderer.requestFrame()

  // optional: defaults to empty function
  before_render: () => {
    // if you want to synchronize the scene state
    // with the rendering loop, use this callback
    // to update scene objects right before render
  },

  // optional: defaults to empty function
  after_render: (frame) => {
    // frame is an ImageBitmap instance.
    // Note that ImageBitmap is immutable. You
    // have to copy it if you want to manipulate it.
  },
});

// Can be called at any time.
// Returns a Promise of a single frame
let frame = renderer.requestFrame();

// Create a display entity with a video texture
const video = document.getElementById("video");
const videoBackground = new Background({ source: video }); // fullscreen by default
// Supports: blob, ImageBitmap, ImageData, HTMLVideoElement, VideoFrame, HTMLCanvasElement, OffscreenCanvas, URL
// example: const videoBackground = new Background({ source: "https://example.com/video.mp4" });
// Options: flip, opacity, undistorted, hidden, group

// Background is a subclass of Billboard with position locked to (0, 0, camera_far_plane)
// Billboard is a subclass of Sprite with rotation locked to the viewer

scene.add(videoBackground);

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

// Starts the rendering loop.
// You can pass configurations here as well:
renderer.render({
  framerate: 60,
});

// you can pass configuration here as well.

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
