# Rendering Targets

## Decision

File Targets (for video and image sequence) have been deprecated in favor of a `post_rendering:` callback property in the **Target** object. This property accepts a user-defined function and will call it every frame with the last rendered image.

Additionally, the **Target** object has an ad-hoc `request_frame()` function that returns the next rendered frame. This can be useful for usecases where the user needs a custom framerate.

### Update (v0.7.0-rc)

The actual implementation differed from this design document.

The Renderer is now implicit, and is globally available as an internal object.

The settings listed here in the "New API" for the Renderer are all inputs to the scene.addTarget() method.

## Context

The original design of the Target object included a `file` field, which would be a path to a **video file** or **image sequence**.

I decided to remove file targets from the library API, as I realized they are not actually necessary.
All we need is the **callback** target, and the user can save the offscreen frames however they want.

If we need file targets in the API, we can add them later as an **extension** to the lib, but not as part of the core API.

## Old API design

```Javascript
scene.addTarget({ type: "canvas", selector: "#output_canvas" });
// supports: canvas (web only), window (native only), file (native only), callback

scene.addCanvasTarget({ selector: "#output_canvas" });
scene.addWindowTarget({ height: 1000, width: 800, name: "Window Title" });
scene.addCallbackTarget({
  height: 1000,
  width: 800,
  callback: (image_rgb) => {
    console.log(image_rgb);
  },
});
```

## New API design

```Javascript
// The Scene is a container of renderable entities
// It manages the spatial relationship between them
const scene = plr.Scene();

// Targets are platform-specific surfaces.
const canvas = document.getElementById("output_canvas");

// A Renderer draws a scene to one or multiple targets
const renderer = plr.Renderer({
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

// can be requested at any time
renderer.requestFrame(); // returns a Promise of a single frame
```
