# Rendering Targets

## Decision

File Targets (for video and image sequence) have been deprecated in favor of a `post_rendering:` callback property in the **Target** object. This property accepts a user-defined function and will call it every frame with the last rendered image.

Additionally, the **Target** object has an ad-hoc `request_frame()` function that returns the next rendered frame.

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
