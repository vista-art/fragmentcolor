import { positionForTime, undistortParams } from "./mocks.js";
import load_wasm, {
  Scene,
  Target,
  Renderer,
  Sprite,
  Circle,
} from "../pkg/plrender.js";
await load_wasm();

// The Scene is a container of renderable entities.
// It manages the spatial relationship between them.
// This is where we add the objects we want to draw.
const scene = new Scene();

// A platform-specific surface where we will draw the scene.
// Examples: OS window, Jupyter cell, Web Canvas, Texture
const canvas = document.getElementById("output_canvas");

// The Target defines how a platform-specific surface is rendered.
// You don't need to create it explicitly: the renderer will create
// one for you with a fullscreen region and a transparent background.
let target = scene.addTarget({
  // optional: a platform-specific drawing surface.
  //           if no target is set, you can read the
  //           rendered image from the afterRender callback
  // supports: QuerySelector, CanvasElement, OffscreenCanvas
  target: canvas, // or "#output_canvas"

  // optional: defaults to target.width
  //           or 800 if no target is set
  width: canvas.width,

  // optional: defaults to target.height
  //           or 600 if no target is set
  height: canvas.height,

  // optional: defaults to fully transparent
  clear_color: "#00000000", // supports any CSS color string

  // optional: defaults to fullscreen (surface size).
  // this is the region of the target to draw the scene
  // you can use it to draw multiple scenes to the same element
  region: {
    x: 0,
    y: 0,
    width: canvas.width,
    height: canvas.height,
  },

  // optional: defaults to screen's refresh rate, normally 60 hz
  // In the web, this is the same frequency as requestAnimationFrame()
  target_fps: 60,

  // optional: defaults to empty function
  beforeRender: () => {
    // if you want to synchronize the scene state
    // with the rendering loop, use this callback
    // to update scene objects right before render
  },

  // optional: defaults to empty function
  afterRender: (frame) => {
    // frame is an ImageBitmap instance.
    // Note that ImageBitmap is immutable. You
    // have to copy it if you want to manipulate it.
  },
});

// A Renderer draws a scene to one or multiple targets
// const renderer = new Renderer({
//   // optional: defaults to empty array []
//   // supports: plrender.Target, QuerySelector, CanvasElement, OffscreenCanvas
//   // if a plrender.Target is not explicitly passed, the renderer will create one
//   targets: [target],

//   log_level: "info", // or 'trace', 'debug', 'warn', 'error'

//   power_preference: "high-performance", // or 'low-power', 'no-preference'

//   force_software_rendering: false,
// });

// or:
// renderer.addTarget(target);

// Create a video entity
const video = document.getElementById("video");

// fullscreen by default
// Supports: blob, ImageBitmap, ImageData, HTMLVideoElement, VideoFrame, HTMLCanvasElement, OffscreenCanvas, URL
// example: const videoBackground = new Background({ source: "https://example.com/video.mp4" });
// Options: flip, opacity, undistorted, hidden, group

// Background is a subclass of Billboard with position locked to (0, 0, camera_far_plane)
// Billboard is a subclass of Sprite with rotation locked to the viewer

const background = new Sprite({
  source: video,
});

// applies non-destructive lens correction
const { camera_matrix, distortion_coefficients } = undistortParams();
background.undistort({ camera_matrix, distortion_coefficients });
background.undistorted = false; // disables undistortion, displays original distorted image
background.undistorted = true; // enables undistortion, displays corrected image (original is preserved)

scene.add(background);

// The circle renders on top of the video because it is created later
const gaze = new Circle({
  color: "#ff000088",
  radius: 0.05,
  border: 0.01,
  position: { x: canvas.width / 2, y: canvas.height / 2 },
});

scene.add(gaze);

// undistorts position only, does not warp the circle
gaze.undistortPosition({
  camera_matrix,
  distortion_coefficients,
});

const playback = new Text({
  text: "00:00",
  color: "#ffffff",
  font: "bold 48px sans-serif",

  position: { x: canvas.width - 10, y: canvas.height - 10 },
});

scene.add(text);

// Updates gaze position every video frame.
function updateLoop() {
  const currentTime = video.currentTime;
  const minutes = Math.floor(time / 60);
  const seconds = Math.floor(time - minutes * 60);
  const timeString = `${minutes}:${seconds}`;
  const { x, y } = positionForTime(currentTime);

  gaze.setPosition(x, y);
  background.update(video);
  playback.setText(timeString);

  scene.render(scene, target);

  video.requestVideoFrameCallback(updateLoop);
}

// Actual video frame rate, Chrome/Webkit only.
// Alternatively, use requestAnimationFrame(updateLoop) for 60fps.
video.requestVideoFrameCallback(updateLoop);
